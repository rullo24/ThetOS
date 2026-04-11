// core imports
use core::sync::atomic::{AtomicU32, Ordering};
use core::ptr::{null_mut, addr_of_mut};
use core::ops::FnOnce;
use core::result::Result;

// local imports
use kernel::{Kernel, KernelStackResources};
use specs::arch::{
    ContextSwitch, 
    ContextSwitchError,
    StackGuard,
    StackGuardContext,
    StackGuardError,
    StackGuardMode,
    StackGuardState,
};
use specs::common::TaskId;
use specs::kernel::{CriticalSection, SchedulerPolicy, KernelError};

// global var to track num of times context switch is triggered
static CTX_SWITCH_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);

// global var to hold stack pool for testing
static mut POOL_KERNEL_INIT: [u8; 1024] = [0; 1024];
static mut POOL_SPAWN_OK: [u8; 1024] = [0; 1024];
static mut POOL_SPAWN_REJECT: [u8; 1024] = [0; 1024];
static mut POOL_CRIT: [u8; 1024] = [0; 1024];
static mut POOL_YIELD: [u8; 1024] = [0; 1024];

// required for testing static slot tables
const TEST_MAX_TASKS: usize = 32;
static mut MOCK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; TEST_MAX_TASKS] = [None; TEST_MAX_TASKS];

#[derive(Clone, Copy)]
struct MockStackGuard;

impl StackGuard for MockStackGuard {

    /// DESCRIPTION
    /// initialise stack guard metadata and seed canary/watermark state.
    fn initialise(
        &self,
        ctx: &mut StackGuardContext,
    ) -> Result<StackGuardState, StackGuardError> {
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

    /// DESCRIPTION
    /// verify stack guard integrity for canary/watermark mode.
    fn check(   
        &self,
        ctx: &mut StackGuardContext,
    ) -> Result<(), StackGuardError> {
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
struct MockContextSwitch;

/// DESCRIPTION
/// mock context switch implementation that increments global test counter on trigger
impl ContextSwitch for MockContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize = 8;
    type TaskContext = usize; // dummy type to test specs/kernel setup (pre-logic)

    /// DESCRIPTION
    /// initialise task context w/ dummy value (nothing to run)
    fn initialise_task_context(
        &self,
        _stack_top: *mut u8,
        _stack_limit: *mut u8,
        _entry_point: extern "C" fn(*mut ()) -> !,
        _entry_arg: *mut (),
    ) -> Result<Self::TaskContext, ContextSwitchError> {
        Ok(_stack_top as usize) // return dummy value
    }

    /// DESCRIPTION
    /// increment global test counter to track num of times context switch is triggered
    fn trigger_pendsv_switch(&self) {
        CTX_SWITCH_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

struct MockCriticalSection;

struct MockScheduler;

impl SchedulerPolicy for MockScheduler {
    fn on_task_spawn(&mut self, _task_id: TaskId) {
        // do nothing...
    }
}

/// DESCRIPTION
/// mock critical section implementation that calls operation parsed
impl CriticalSection for MockCriticalSection {
    fn with_execute<Res, Op>(&self, operation: Op) -> Res 
    where Op: FnOnce() -> Res,
    {
        operation()
    }
}

/// DESCRIPTION
/// dummy entry point for task (run nothing)
extern "C" fn dummy_entry(_arg: *mut ()) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// DESCRIPTION
/// create a new kernel stack resources instance for testing
fn test_resources(pool: &'static mut [u8]) -> KernelStackResources<MockStackGuard> {
    KernelStackResources::new(
        pool,
        MockStackGuard,
        unsafe { &mut *addr_of_mut!(MOCK_STACK_GUARD_SLOTS) },
    )
}

/////////////
// TESTING //
/////////////

#[test]
fn kernel_init_with_mocks() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_KERNEL_INIT) };
    let kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
    );

    assert_eq!(kernel.get_task_count(), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn spawn_task_registers_first_task() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_OK) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
    );

    let result = kernel.spawn_task(
        TaskId(1),
        1024,
        dummy_entry, // does nothing
        null_mut(), // no argument
    );

    assert!(result.is_ok());
    assert_eq!(kernel.get_task_count(), 1);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));

}

#[test]
fn spawn_task_rejects_null_stack_top() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_REJECT) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
    );

    let result = kernel.spawn_task(
        TaskId(1),
        32, // too small stack size
        dummy_entry, // does nothing
        null_mut(), // no argument
    );

    assert_eq!(result, Err(KernelError::InvalidConfig));
    assert_eq!(kernel.get_task_count(), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn execute_in_critical_section_runs_operation() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_CRIT) };
    let kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
    );

    let value: usize = kernel.execute_in_critical_section(|| 42);

    assert_eq!(value, 42);
}

#[test]
fn yield_now_triggers_ctx_switch() {

    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_YIELD) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
    );

    kernel.yield_now();

    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 1);

}