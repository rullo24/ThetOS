use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::Ordering;
use kernel::scheduler::FppScheduler;
use kernel::Kernel;
use specs::common::TaskId;
use specs::kernel::{KernelError, TaskPriority, TickAction};

mod support;

use support::{
    dummy_entry, test_resources, MockContextSwitch, MockCriticalSection, MockScheduler,
    CTX_SWITCH_TRIGGER_COUNT, POOL_CRIT, POOL_KERNEL_INIT, POOL_QUEUE_FULL, POOL_SPAWN_OK,
    POOL_SPAWN_REJECT, POOL_TICK_NO_ACTION, POOL_TICK_NO_TASKS, POOL_TICK_SINGLE_TASK,
    POOL_TICK_SWITCH, POOL_YIELD, POOL_YIELD_SAME_PRIORITY,
};

use crate::support::MockSystemTimer;

#[test]
fn kernel_init_with_mocks() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_KERNEL_INIT) };
    let kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::None,
        },
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
        MockSystemTimer {
            next_action: TickAction::None,
        },
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
        MockSystemTimer {
            next_action: TickAction::None,
        },
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
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::None,
        },
    );

    let value: usize = kernel.execute_in_critical_section(|_kernel| 42);
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
        MockSystemTimer {
            next_action: TickAction::None,
        },
    );

    kernel.yield_now().unwrap();
    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn on_tick_interrupt_triggers_switch_when_task_changes() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_SWITCH) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::RequestReschedule,
        },
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(20).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), TaskPriority::new(5).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));

    kernel.on_tick_interrupt().unwrap();

    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
}

#[test]
fn on_tick_interrupt_does_not_trigger_switch_when_no_action_requested() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_NO_ACTION) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::None,
        },
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(10).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), TaskPriority::new(5).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    kernel.on_tick_interrupt().unwrap();

    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 0);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
}

#[test]
fn on_tick_interrupt_does_not_trigger_switch_when_only_task_reselects_itself() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_SINGLE_TASK) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::RequestReschedule,
        },
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(10).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    kernel.on_tick_interrupt().unwrap();

    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 0);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
}

#[test]
fn on_tick_interrupt_with_no_tasks_is_a_safe_no_op() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_NO_TASKS) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::RequestReschedule,
        },
    );

    let result = kernel.on_tick_interrupt();

    assert!(result.is_ok());
    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn spawn_task_propagates_ready_queue_full_error() {
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_QUEUE_FULL) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::None,
        },
    );

    let same_priority = TaskPriority::new(15).unwrap();

    // READY_QUEUE_CAPACITY is 8 per priority level. the first spawn becomes
    // curr_task directly and is not enqueued (see spawn_task), so it takes
    // 9 spawns to fill the queue (1 not enqueued + 8 enqueued) before the
    // 10th same-priority spawn overflows it.
    for i in 0..9u32 {
        let result = kernel.spawn_task(
            TaskId(i),
            same_priority,
            1024,
            dummy_entry,
            null_mut(),
        );
        assert!(result.is_ok(), "spawn {i} unexpectedly failed: {result:?}");
    }

    let result = kernel.spawn_task(
        TaskId(9),
        same_priority,
        1024,
        dummy_entry,
        null_mut(),
    );

    assert_eq!(result, Err(KernelError::ReadyQueueFull));
}

#[test]
fn yield_now_same_priority_advances_to_next_task() {
    // spawn_task no longer enqueues the task that becomes curr_task directly,
    // so it isn't double-booked as both running and ready -> the first
    // yield_now() at the same priority correctly advances to the next task
    // instead of reselecting the one that just spawned.
    CTX_SWITCH_TRIGGER_COUNT.store(0, Ordering::SeqCst);

    let pool = unsafe { &mut *addr_of_mut!(POOL_YIELD_SAME_PRIORITY) };
    let mut kernel = Kernel::new(
        MockContextSwitch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        MockSystemTimer {
            next_action: TickAction::None,
        },
    );

    let priority = TaskPriority::new(12).unwrap();
    kernel
        .spawn_task(TaskId(1), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));

    // first yield: advances straight to TaskId(2), no wasted self-reselect.
    kernel.yield_now().unwrap();
    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));

    // second yield: cycles back to TaskId(1).
    kernel.yield_now().unwrap();
    assert_eq!(CTX_SWITCH_TRIGGER_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
}
