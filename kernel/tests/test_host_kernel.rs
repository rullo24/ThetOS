// core imports
use core::sync::atomic::{AtomicU32, Ordering};
use core::ptr::null_mut;
use core::ops::FnOnce;

// local imports
use kernel::Kernel;
use specs::arch::ContextSwitch;
use specs::common::{TaskId, ThetosError};
use specs::kernel::{CriticalSection, SchedulerPolicy};

// global var to track num of times context switch is triggered
static CTX_SWITCH_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);

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
        stack_top: *mut u8,
        _entry_point: extern "C" fn(*mut ()) -> !,
        _entry_arg: *mut (),
    ) -> Self::TaskContext {
        return stack_top as usize; // return dummy value
    }

    /// DESCRIPTION
    /// increment global test counter to track num of times context switch is triggered
    fn trigger_pend_switch(&self) {
        CTX_SWITCH_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

struct MockCriticalSection;

struct MockScheduler;

impl SchedulerPolicy for MockScheduler {
    fn on_task_spawn(&mut self, _task_id: TaskId) {}
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

/////////////
// TESTING //
/////////////

#[test]
fn kernel_init_with_mocks() {
    let ctx_switch = MockContextSwitch;
    let crit_section = MockCriticalSection;
    let scheduler = MockScheduler;
    let kernel = Kernel::new(ctx_switch, crit_section, scheduler);

    assert_eq!(kernel.get_task_count(), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn spawn_task_registers_first_task() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let ctx_switch = MockContextSwitch;
    let crit_section = MockCriticalSection;
    let scheduler = MockScheduler;
    let mut kernel = Kernel::new(ctx_switch, crit_section, scheduler);

    let result = kernel.spawn_task(
        TaskId(1),
        0x1000 as *mut u8, // dummy stack top
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

    let ctx_switch = MockContextSwitch;
    let crit_section = MockCriticalSection;
    let scheduler = MockScheduler;
    let mut kernel = Kernel::new(ctx_switch, crit_section, scheduler);

    let result = kernel.spawn_task(
        TaskId(1),
        null_mut(), // null stack top
        dummy_entry, // does nothing
        null_mut(), // no argument
    );

    assert_eq!(result, Err(ThetosError::InvalidConfig));
    assert_eq!(kernel.get_task_count(), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn execute_in_critical_section_runs_operation() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let ctx_switch = MockContextSwitch;
    let crit_section = MockCriticalSection;
    let scheduler = MockScheduler;
    let kernel = Kernel::new(ctx_switch, crit_section, scheduler);

    let value: usize = kernel.execute_in_critical_section(|| 42);

    assert_eq!(value, 42);
}

#[test]
fn yield_now_triggers_ctx_switch() {

    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let ctx_switch = MockContextSwitch;
    let crit_section = MockCriticalSection;
    let scheduler = MockScheduler;
    let kernel = Kernel::new(ctx_switch, crit_section, scheduler);

    kernel.yield_now();

    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 1);

}