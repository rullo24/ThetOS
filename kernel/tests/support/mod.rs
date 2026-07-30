use core::ops::FnOnce;
use core::ptr::addr_of_mut;
use core::result::Result;
use core::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use kernel::KernelStackResources;
use specs::arch::{
    ContextSwitch, ContextSwitchError, StackGuard, StackGuardContext, StackGuardError,
    StackGuardMode, StackGuardState,
};
use specs::common::TaskId;
use specs::kernel::{
    CriticalSection, KernelError, SchedulerPolicy, SystemTimer, TaskPriority, TickAction,
};

pub static mut POOL_KERNEL_INIT: [u8; 1024] = [0; 1024];
pub static mut POOL_SPAWN_OK: [u8; 1024] = [0; 1024];
pub static mut POOL_SPAWN_REJECT: [u8; 1024] = [0; 1024];
pub static mut POOL_CRIT: [u8; 1024] = [0; 1024];
pub static mut POOL_YIELD: [u8; 1024] = [0; 1024];
pub static mut POOL_TICK_SWITCH: [u8; 4096] = [0; 4096];
pub static mut POOL_TICK_NO_ACTION: [u8; 4096] = [0; 4096];
pub static mut POOL_TICK_SINGLE_TASK: [u8; 1024] = [0; 1024];
pub static mut POOL_TICK_NO_TASKS: [u8; 1024] = [0; 1024];
pub static mut POOL_QUEUE_FULL: [u8; 16384] = [0; 16384];
pub static mut POOL_YIELD_SAME_PRIORITY: [u8; 4096] = [0; 4096];
pub static mut POOL_TICK_ACK_ON_ERROR: [u8; 16384] = [0; 16384];
pub static mut POOL_SPAWN_PREEMPT: [u8; 4096] = [0; 4096];
pub static mut POOL_SPAWN_NO_PREEMPT: [u8; 4096] = [0; 4096];
pub static mut POOL_SPAWN_PREEMPT_REQUEUE_FULL: [u8; 16384] = [0; 16384];

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

#[derive(Clone)]
pub struct MockContextSwitch {
    pub trigger_count: Arc<AtomicU32>,
}

impl MockContextSwitch {
    /// DESCRIPTION
    /// create a mock w/ its own private trigger counter -> avoids sharing
    /// mutable state across tests running in parallel (cargo test's default).
    pub fn new() -> (Self, Arc<AtomicU32>) {
        let trigger_count = Arc::new(AtomicU32::new(0));
        (Self { trigger_count: trigger_count.clone() }, trigger_count)
    }
}

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
        self.trigger_count.fetch_add(1, Ordering::SeqCst);
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

pub struct MockSystemTimer {
    pub next_action: TickAction,
    pub ack_count: Arc<AtomicU32>,
}

impl MockSystemTimer {
    /// DESCRIPTION
    /// create a mock w/ its own private ack counter -> avoids sharing
    /// mutable state across tests running in parallel (cargo test's default).
    pub fn new(next_action: TickAction) -> (Self, Arc<AtomicU32>) {
        let ack_count = Arc::new(AtomicU32::new(0));
        (
            Self { next_action, ack_count: ack_count.clone() },
            ack_count,
        )
    }
}

impl SystemTimer for MockSystemTimer {
    type Error = ();

    fn initialise(&mut self, _reload_ticks: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn acknowledge_tick_interrupt(&mut self) -> Result<(), Self::Error> {
        self.ack_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn on_tick_interrupt(&mut self) -> Result<TickAction, Self::Error> {
        Ok(self.next_action)
    }
}
