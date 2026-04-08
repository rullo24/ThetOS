#![no_std]

pub mod tcb;

// local imports
use specs::arch::ContextSwitch;
use specs::common::{Result, TaskId, ThetosError};
use specs::kernel::{CriticalSection, SchedulerPolicy};

// must cover at least the initial task frame (hw + callee) and any padding
const MIN_TASK_STACK_SIZE_BYTES: usize = 512; // arbitrary min size to cover all targets

#[inline]
fn align_up(n: usize, align: usize) -> usize {
    debug_assert!(align > 0);
    return (n + align - 1) / align * align;
}

// hardware-blind kernel
pub struct Kernel<CtxSwitchType, CriticalSectionType, SchedulerType>
where
    CtxSwitchType: ContextSwitch,
    CriticalSectionType: CriticalSection,
    SchedulerType: SchedulerPolicy,
{
    ctx_switch: CtxSwitchType,
    crit_section: CriticalSectionType,
    scheduler: SchedulerType,
    curr_task: Option<TaskId>,
    task_count: usize,
    stack_pool: &'static mut [u8], // pool for task alloc
    stack_cursor: usize, // next free stack addr in pool
}

impl<CtxSwitchType, CriticalSectionType, SchedulerType> Kernel<CtxSwitchType, CriticalSectionType, SchedulerType>
where
    CtxSwitchType: ContextSwitch,
    CriticalSectionType: CriticalSection,
    SchedulerType: SchedulerPolicy,
{
    /// DESCRIPTION
    /// create a new hardware-blind kernel instance
    pub fn new(
        ctx_switch: CtxSwitchType, 
        crit_section: CriticalSectionType, 
        scheduler: SchedulerType,
        stack_pool: &'static mut [u8],
    ) -> Self {
        return Self {
            ctx_switch,
            crit_section,
            scheduler,
            curr_task: None,
            task_count: 0,
            stack_pool,
            stack_cursor: 0x0,
        };
    }

    /// DESCRIPTION
    /// spawn a new task w/ registered context
    pub fn spawn_task(
        &mut self,
        task_id: TaskId,
        stack_size: usize,
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Result<()> {
        
        // read arch-specific stack alignment -> reject if zero (invalid)
        let align = CtxSwitchType::STACK_ALIGNMENT_BYTES;
        if align == 0 {
            return Err(ThetosError::InvalidConfig);
        }

        // round stack_size up to arch alignment for valid allocation
        let aligned_size = align_up(stack_size, align);
        if aligned_size < MIN_TASK_STACK_SIZE_BYTES {
            return Err(ThetosError::InvalidConfig);
        }

        // align bump cursor so stack_limit is on alignment boundary
        let cursor_aligned = align_up(self.stack_cursor, align);
        if cursor_aligned
            .checked_add(aligned_size)
            .map_or(true, |end| end > self.stack_pool.len())
        {
            return Err(ThetosError::InvalidConfig);
        }

        // capture limit + top from stack pool and advance cursor for next spawn
        let stack_limit = self.stack_pool.as_mut_ptr().wrapping_add(cursor_aligned);
        let stack_top = stack_limit.wrapping_add(aligned_size);
        self.stack_cursor = cursor_aligned + aligned_size;

        // initialise task context for the new task
        let _ = self.ctx_switch.initialise_task_context(stack_top, stack_limit, entry_point, entry_arg);
        self.scheduler.on_task_spawn(task_id);
        self.task_count += 1; 

        // checking if no task is currently running
        if self.curr_task.is_none() {
            self.curr_task = Some(task_id); // setting the new task as the current task
        }

        return Ok(()); // success
    }


    /// DESCRIPTION
    /// trigger a voluntary context switch
    pub fn yield_now(&self) {
        self.ctx_switch.trigger_pendsv_switch();
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
    pub fn execute_in_critical_section<Res, Op>(&self, operation: Op) -> Res 
    where Op: FnOnce() -> Res, // called at least once before return
    {
        return self.crit_section.with_execute(operation);
    }

}