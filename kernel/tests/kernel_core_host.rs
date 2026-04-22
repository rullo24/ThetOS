use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::Ordering;
use kernel::Kernel;
use specs::common::TaskId;
use specs::kernel::{KernelError, TaskPriority};

mod support;

use support::{
    dummy_entry, test_resources, MockContextSwitch, 
    MockCriticalSection, MockScheduler, CTX_SWITCH_TRIGGER_COUNT, 
    POOL_CRIT, POOL_KERNEL_INIT, POOL_SPAWN_OK, 
    POOL_SPAWN_REJECT, POOL_YIELD,
};

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
        TaskPriority::new(1).unwrap(),
        1024,
        dummy_entry,
        null_mut(),
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
        TaskPriority::new(1).unwrap(),
        32,
        dummy_entry,
        null_mut(),
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

    kernel.yield_now().unwrap();
    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 1);
}
