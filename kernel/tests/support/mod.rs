use core::ops::FnOnce;
use core::ptr::addr_of_mut;
use core::result::Result;
use core::sync::atomic::{AtomicU32, Ordering};
use kernel::KernelStackResources;
use specs::arch::{
    ContextSwitch, ContextSwitchError, StackGuard, StackGuardContext, StackGuardError,
    StackGuardMode, StackGuardState,
};
use specs::common::TaskId;
use specs::kernel::{CriticalSection, KernelError, SchedulerPolicy, TaskPriority};

pub static CTX_SWITCH_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
pub static mut POOL_KERNEL_INIT: [u8; 1024] = [0; 1024];
pub static mut POOL_SPAWN_OK: [u8; 1024] = [0; 1024];
pub static mut POOL_SPAWN_REJECT: [u8; 1024] = [0; 1024];
pub static mut POOL_CRIT: [u8; 1024] = [0; 1024];
pub static mut POOL_YIELD: [u8; 1024] = [0; 1024];

const TEST_MAX_TASKS: usize = 32;
static mut MOCK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; TEST_MAX_TASKS] =
    [None; TEST_MAX_TASKS];

#[derive(Clone, Copy)]
pub struct MockStackGuard;

impl StackGuard for MockStackGuard {
    fn initialise(&self, ctx: &mut StackGuardContext) -> Result<StackGuardState, StackGuardError> {
        if ctx.stack_limit.is_null() || ctx.stack_top.is_null() {
            return Err(StackGuardError::InvalidStackBounds);
        }
        if (ctx.stack_top as usize) <= (ctx.stack_limit as usize) {
            return Err(StackGuardError::InvalidStackBounds);
        }
        if matches!(ctx.config.mode, StackGuardMode::Canary) {
            unsafe {
                (ctx.stack_limit as *mut u32).write_volatile(ctx.config.canary_word);
            }
        }
        ctx.state.low_mark = ctx.stack_limit;
        Ok(ctx.state)
    }

    fn check(&self, ctx: &mut StackGuardContext) -> Result<(), StackGuardError> {
        if matches!(ctx.config.mode, StackGuardMode::Canary) {
            let v = unsafe { (ctx.stack_limit as *const u32).read_volatile() };
            if v != ctx.config.canary_word {
                return Err(StackGuardError::GuardCorrupted);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct MockContextSwitch;

impl ContextSwitch for MockContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize = 8;
    type TaskContext = usize;

    fn initialise_task_context(
        &self,
        stack_top: *mut u8,
        _stack_limit: *mut u8,
        _entry_point: extern "C" fn(*mut ()) -> !,
        _entry_arg: *mut (),
    ) -> Result<Self::TaskContext, ContextSwitchError> {
        Ok(stack_top as usize)
    }

    fn trigger_pendsv_switch(&self) {
        CTX_SWITCH_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy)]
pub struct MockCriticalSection;

impl CriticalSection for MockCriticalSection {
    fn with_execute<Res, Op>(&self, operation: Op) -> Res
    where
        Op: FnOnce() -> Res,
    {
        operation()
    }
}

pub struct MockScheduler;

impl SchedulerPolicy for MockScheduler {
    fn register_task(
        &mut self,
        _task_id: TaskId,
        _priority: TaskPriority,
    ) -> Result<(), KernelError> {
        Ok(())
    }

    fn enqueue_runnable(
        &mut self,
        _task_id: TaskId,
        _priority: TaskPriority,
    ) -> Result<(), KernelError> {
        Ok(())
    }

    fn select_next_runnable(&mut self) -> Option<TaskId> {
        None
    }

    fn should_preempt_current(
        &self,
        _current: Option<(TaskId, TaskPriority)>,
        _candidate: (TaskId, TaskPriority),
    ) -> bool {
        false
    }
}

pub extern "C" fn dummy_entry(_arg: *mut ()) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn test_resources(pool: &'static mut [u8]) -> KernelStackResources<MockStackGuard> {
    KernelStackResources::new(pool, MockStackGuard, unsafe {
        &mut *addr_of_mut!(MOCK_STACK_GUARD_SLOTS)
    })
}
