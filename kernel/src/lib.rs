#![no_std]

pub mod scheduler;
pub mod stack_resources;
pub mod tcb;

// local imports
use crate::tcb::TaskControlBlock;
pub use scheduler::FppScheduler;
use specs::arch::{
    ContextSwitch, ContextSwitchError, StackGuard, StackGuardConfig, StackGuardContext,
    StackGuardError, StackGuardMode, StackGuardState,
};
use specs::common::TaskId;
use specs::kernel::{
    CoreTcb, CriticalSection, KernelError, Result, SchedulerPolicy, SystemTimer, TaskPriority,
    TaskState, TickAction,
};
pub use stack_resources::KernelStackResources;

// constants
const MAX_TASKS: usize = 32;

// must cover at least the initial task frame (hw + callee) and any padding
const MIN_TASK_STACK_SIZE_BYTES: usize = 512; // arbitrary min size to cover all targets
const DEFAULT_STACK_CANARY_WORD: u32 = 0xDEADBEEF;

/// DESCRIPTION
/// align a value up to the nearest multiple of the alignment
#[inline]
fn align_up(n: usize, align: usize) -> usize {
    debug_assert!(align > 0);
    return (n + align - 1) / align * align;
}

/// DESCRIPTION
/// build a default stack guard context
#[inline]
fn build_default_stack_guard_ctx(
    stack_top: *mut u8,
    stack_limit: *mut u8,
) -> Result<StackGuardContext> {
    // check that stack ptrs make sense
    if (stack_top as usize) <= (stack_limit as usize) {
        return Err(KernelError::InvalidConfig);
    }

    Ok(StackGuardContext {
        stack_top,
        stack_limit,
        state: StackGuardState {
            low_mark: stack_limit,
        },
        config: StackGuardConfig {
            mode: StackGuardMode::Canary,
            canary_word: DEFAULT_STACK_CANARY_WORD,
        },
    })
}

/// DESCRIPTION
/// map context switch errors to kernel errors
fn map_ctx_switch_err_to_kernel_err(err: ContextSwitchError) -> KernelError {
    match err {
        ContextSwitchError::NullStackPointer => KernelError::InvalidConfig,
        ContextSwitchError::InvalidStackBounds => KernelError::InvalidConfig,
        ContextSwitchError::UnalignedStackTop => KernelError::InvalidConfig,
        ContextSwitchError::StackRegionTooSmall => KernelError::InvalidConfig,
        ContextSwitchError::InvalidEntryPoint => KernelError::InvalidConfig,
        _ => KernelError::Unsupported,
    }
}

/// DESCRIPTION
/// map stack guard errors to kernel errors
fn map_stack_guard_err_to_kernel_err(err: StackGuardError) -> KernelError {
    match err {
        StackGuardError::InvalidStackBounds => KernelError::InvalidConfig,
        StackGuardError::GuardCorrupted => KernelError::StackGuard,
    }
}

/// DESCRIPTION
/// map system timer errors to kernel errors
fn map_timer_err_to_kernel_err<E: core::fmt::Debug>(_err: E) -> KernelError {
    KernelError::TimerFault
}

// hardware-blind kernel
pub struct Kernel<
    CtxSwitchType,
    CriticalSectionType,
    SchedulerType,
    StackGuardImpl,
    SystemTimerType,
> where
    CtxSwitchType: ContextSwitch,
    CriticalSectionType: CriticalSection + Copy,
    SchedulerType: SchedulerPolicy,
    StackGuardImpl: StackGuard + Copy,
    SystemTimerType: SystemTimer,
{
    ctx_switch: CtxSwitchType,
    crit_section: CriticalSectionType,
    scheduler: SchedulerType,
    system_timer: SystemTimerType,
    curr_task: Option<TaskId>,
    task_count: usize,
    stack_cursor: usize,                                   // kernel runtime state
    stack_resources: KernelStackResources<StackGuardImpl>, // BSP supplied stack resources
    tcb_list: [Option<TaskControlBlock<CtxSwitchType::TaskContext>>; MAX_TASKS],
}

impl<CtxSwitchType, CriticalSectionType, SchedulerType, StackGuardImpl, SystemTimerType>
    Kernel<CtxSwitchType, CriticalSectionType, SchedulerType, StackGuardImpl, SystemTimerType>
where
    CtxSwitchType: ContextSwitch,
    CriticalSectionType: CriticalSection + Copy,
    SchedulerType: SchedulerPolicy,
    StackGuardImpl: StackGuard + Copy,
    SystemTimerType: SystemTimer,
{
    /// DESCRIPTION
    /// create a new hardware-blind kernel instance
    pub fn new(
        ctx_switch: CtxSwitchType,
        crit_section: CriticalSectionType,
        scheduler: SchedulerType,
        stack_resources: KernelStackResources<StackGuardImpl>,
        system_timer: SystemTimerType,
    ) -> Self {
        return Self {
            ctx_switch,
            crit_section,
            scheduler,
            system_timer,
            curr_task: None,
            task_count: 0,
            stack_cursor: 0x0,
            stack_resources,
            tcb_list: core::array::from_fn(|_| None), // initialise all TCBs to None
        };
    }

    /// DESCRIPTION
    /// spawn a new task w/ registered context
    pub fn spawn_task(
        &mut self,
        task_id: TaskId,
        priority: TaskPriority,
        stack_size: usize,
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Result<()> {
        // use stack_resources for the pool -> checking available slots for new task
        let idx: usize = task_id.0 as usize;
        if idx >= self.stack_resources.stack_guard_slots.len() {
            return Err(KernelError::InvalidConfig);
        }

        // read arch-specific stack alignment -> reject if zero (invalid)
        let align = CtxSwitchType::STACK_ALIGNMENT_BYTES;
        if align == 0 {
            return Err(KernelError::InvalidConfig);
        }

        // round stack_size up to arch alignment for valid allocation
        let aligned_size = align_up(stack_size, align);
        if aligned_size < MIN_TASK_STACK_SIZE_BYTES {
            return Err(KernelError::InvalidConfig);
        }

        // align bump cursor so stack_limit is on alignment boundary
        let cursor_aligned = align_up(self.stack_cursor, align);
        if cursor_aligned
            .checked_add(aligned_size)
            .map_or(true, |end| end > self.stack_resources.stack_pool.len())
        {
            return Err(KernelError::InvalidConfig);
        }

        // capture limit + top from stack pool and advance cursor for next spawn
        let stack_limit = self
            .stack_resources
            .stack_pool
            .as_mut_ptr()
            .wrapping_add(cursor_aligned);
        let stack_top = stack_limit.wrapping_add(aligned_size);

        // initialise task context for the new task
        let task_context = self
            .ctx_switch
            .initialise_task_context(stack_top, stack_limit, entry_point, entry_arg)
            .map_err(map_ctx_switch_err_to_kernel_err)?; // throw error upwards if fails

        // build the stack guard context
        let mut stack_guard_ctx = build_default_stack_guard_ctx(stack_top, stack_limit)?;
        self.stack_resources
            .stack_guard // use BSP supplied stack guard impl
            .initialise(&mut stack_guard_ctx)
            .map_err(map_stack_guard_err_to_kernel_err)?; // throw error upwards if fails
        self.stack_resources.stack_guard_slots[idx] = Some(stack_guard_ctx); // store the stack guard context for the new task

        // a task spawned while nothing else is running becomes curr_task
        // directly; a task spawned while something IS running preempts it
        // only if the scheduler policy says so (should_preempt_current) ->
        // this compares only the new candidate against the current task, so
        // unrelated tasks already sitting in the ready queues are never
        // disturbed by an unrelated spawn.
        let previous_task = self.curr_task; // capture before any mutation below
        let preempt_current = match previous_task {
            None => true,
            Some(current_task_id) => {
                let current_priority = self.tcb_list[current_task_id.0 as usize]
                    .as_ref()
                    .map(|tcb| tcb.get_priority());
                match current_priority {
                    Some(curr_priority) => self.scheduler.should_preempt_current(
                        Some((current_task_id, curr_priority)),
                        (task_id, priority),
                    ),
                    None => false, // no valid current TCB -> nothing to preempt
                }
            }
        };

        // TCB creation
        self.tcb_list[idx] = Some(TaskControlBlock {
            task_id,
            stack_bounds: specs::kernel::StackBounds {
                bottom: stack_limit,
                top: stack_top,
            },
            task_state: if preempt_current {
                TaskState::Running
            } else {
                TaskState::Ready
            },
            task_context, // capture from initialise_task_context
            stack_guard_ctx,
            task_priority: priority,
        });

        // advance cursor for next spawn
        self.stack_cursor = cursor_aligned + aligned_size;

        if preempt_current {
            // demote the previously running task (if any) back to Ready and
            // requeue it -> only this specific task is touched, not a full
            // reschedule() pass over the ready queues.
            if let Some(previous_task_id) = previous_task {
                self.set_task_state(previous_task_id, TaskState::Ready)?;
                if let Some(tcb) = self.tcb_list[previous_task_id.0 as usize].as_ref() {
                    self.scheduler
                        .enqueue_runnable(previous_task_id, tcb.get_priority())?;
                }
            }
            self.curr_task = Some(task_id); // new task preempts and becomes current
        } else {
            self.scheduler.register_task(task_id, priority)?;
        }
        self.task_count += 1;

        // a preemption (including the very first spawn, which preempts
        // "nothing running") must trigger the real context switch, so
        // PendSV actually knows which task to restore.
        if preempt_current {
            // outgoing side: None on the first spawn (nothing to save);
            // PendSV_Handler's asm already skips the save on a null pointer.
            let outgoing_ctx: Option<*mut CtxSwitchType::TaskContext> = previous_task
                .and_then(|id| self.tcb_list[id.0 as usize].as_mut())
                .map(|tcb| tcb.get_context_mut() as *mut _);
            self.ctx_switch.set_current_task_context(outgoing_ctx);

            // incoming side: this spawn's own task, which just became curr_task above.
            if let Some(tcb) = self.tcb_list[idx].as_ref() {
                self.ctx_switch.activate_next_task(tcb.get_context());
            }

            self.ctx_switch.trigger_pendsv_switch();
        }

        return Ok(()); // success
    }

    /// DESCRIPTION
    /// start periodic tick generation; call only once all init (including task spawning) is complete
    pub fn start_system_timer(&mut self) -> Result<()> {
        self.system_timer
            .start()
            .map_err(map_timer_err_to_kernel_err)
    }

    /// DESCRIPTION
    /// handle a system timer tick interrupt. acknowledges the tick before
    /// running any reschedule policy work, so the interrupt source is
    /// cleared even if reschedule() itself errors -> an unacknowledged tick
    /// could otherwise leave the IRQ asserted and re-enter the ISR on return.
    pub fn on_tick_interrupt(&mut self) -> Result<()> {
        let action = self
            .system_timer
            .on_tick_interrupt()
            .map_err(map_timer_err_to_kernel_err)?;

        self.system_timer
            .acknowledge_tick_interrupt()
            .map_err(map_timer_err_to_kernel_err)?;

        if action == TickAction::RequestReschedule {
            let switched = self.execute_in_critical_section(|kernel| kernel.reschedule())?;
            if switched {
                self.ctx_switch.trigger_pendsv_switch();
            }
        }

        Ok(())
    }

    /// DESCRIPTION
    /// requeue the current task (if any) as ready, select the next runnable task, and update kernel state
    fn reschedule(&mut self) -> Result<bool> {
        if let Some(tid) = self.curr_task {
            let i = tid.0 as usize;
            if let Some(ctx) = self
                .stack_resources
                .stack_guard_slots
                .get_mut(i)
                .and_then(|s| s.as_mut())
            {
                if self.stack_resources.stack_guard.check(ctx).is_err() {
                    panic!("stack guard violation detected");
                }
            }
        }

        // current running task -> ready + requeue
        let previous_task = self.curr_task;
        if let Some(current_task_id) = previous_task {
            self.set_task_state(current_task_id, TaskState::Ready)?;
            let idx = current_task_id.0 as usize;
            if let Some(tcb) = self.tcb_list[idx].as_ref() {
                self.scheduler
                    .enqueue_runnable(current_task_id, tcb.get_priority())?;
            }
        }

        // select next runnable task -> running
        if let Some(next_task_id) = self.scheduler.select_next_runnable() {
            self.set_task_state(next_task_id, TaskState::Running)?;
            self.curr_task = Some(next_task_id);
        }

        let switched = self.curr_task != previous_task;
        if switched {
            // outgoing side: None if nothing was running before this reschedule.
            let outgoing_ctx: Option<*mut CtxSwitchType::TaskContext> = previous_task
                .and_then(|id| self.tcb_list[id.0 as usize].as_mut())
                .map(|tcb| tcb.get_context_mut() as *mut _);
            self.ctx_switch.set_current_task_context(outgoing_ctx);

            // incoming side: the task reschedule() just selected above.
            if let Some(next_task_id) = self.curr_task {
                if let Some(tcb) = self.tcb_list[next_task_id.0 as usize].as_ref() {
                    self.ctx_switch.activate_next_task(tcb.get_context());
                }
            }
        }

        Ok(switched)
    }

    /// DESCRIPTION
    /// trigger a voluntary context switch
    pub fn yield_now(&mut self) -> Result<()> {
        self.reschedule()?;

        // trigger context switch
        self.ctx_switch.trigger_pendsv_switch();
        Ok(()) // success
    }

    /// DESCRIPTION
    /// get the currently processes task
    pub fn get_current_task(&self) -> Option<TaskId> {
        return self.curr_task;
    }

    /// DESCRIPTION
    /// return the num of tasks registered
    pub fn get_task_count(&self) -> usize {
        return self.task_count;
    }

    /// DESCRIPTION
    /// execute an operation inside a critical section.
    pub fn execute_in_critical_section<Res, Op>(&mut self, operation: Op) -> Res
    where
        Op: FnOnce(&mut Self) -> Res, // called at least once before return
    {
        let crit = self.crit_section;
        return crit.with_execute(|| operation(self));
    }

    /// DESCRIPTION
    /// update task state
    fn set_task_state(&mut self, task_id: TaskId, state: TaskState) -> Result<()> {
        let idx = task_id.0 as usize;
        let tcb = self.tcb_list[idx]
            .as_mut()
            .ok_or(KernelError::InvalidState)?; // capture TCB or error if invalid
        tcb.set_state(state); // update task state
        Ok(())
    }
}
