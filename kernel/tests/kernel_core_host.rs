use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::Ordering;
use kernel::scheduler::FppScheduler;
use kernel::Kernel;
use specs::common::TaskId;
use specs::kernel::{KernelError, TaskPriority, TickAction};

mod support;

use support::{
    dummy_entry, test_resources, MockContextSwitch, MockCriticalSection, MockScheduler,
    POOL_CRIT, POOL_KERNEL_INIT, POOL_QUEUE_FULL, POOL_SPAWN_FIRST_TASK_CONTEXT,
    POOL_SPAWN_NO_PREEMPT, POOL_SPAWN_OK, POOL_SPAWN_PREEMPT, POOL_SPAWN_PREEMPT_CONTEXT,
    POOL_SPAWN_PREEMPT_REQUEUE_FULL, POOL_SPAWN_REJECT, POOL_TICK_ACK_ON_ERROR,
    POOL_TICK_NO_ACTION, POOL_TICK_NO_TASKS, POOL_TICK_SINGLE_TASK, POOL_TICK_SWITCH,
    POOL_TICK_SWITCH_CONTEXT, POOL_YIELD, POOL_YIELD_SAME_PRIORITY,
};

use crate::support::MockSystemTimer;

#[test]
fn kernel_init_with_mocks() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_KERNEL_INIT) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
    );

    assert_eq!(kernel.get_task_count(), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn spawn_task_registers_first_task() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_OK) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
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
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_REJECT) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
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
    let pool = unsafe { &mut *addr_of_mut!(POOL_CRIT) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
    );

    let value: usize = kernel.execute_in_critical_section(|_kernel| 42);
    assert_eq!(value, 42);
}

#[test]
fn yield_now_triggers_ctx_switch() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_YIELD) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
    );

    kernel.yield_now().unwrap();
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
}

#[test]
fn on_tick_interrupt_triggers_switch_when_task_changes() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_SWITCH) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::RequestReschedule);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    // same priority -> spawning TaskId(2) does not preempt TaskId(1) (see
    // spawn_task's should_preempt_current check); the tick-driven reschedule
    // is what advances to the next task here, isolating tick behaviour from
    // spawn-time preemption.
    let priority = TaskPriority::new(10).unwrap();
    kernel
        .spawn_task(TaskId(1), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
    // the first spawn itself now triggers a switch too (it's what actually
    // enters the first task -> see spawn_task's preempt_current gate).
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);

    kernel.on_tick_interrupt().unwrap();

    assert_eq!(trigger_count.load(Ordering::SeqCst), 2);
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
}

#[test]
fn on_tick_interrupt_does_not_trigger_switch_when_no_action_requested() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_NO_ACTION) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    // same priority -> spawning TaskId(2) does not preempt TaskId(1),
    // isolating this test's "TickAction::None means no switch" claim from
    // spawn-time preemption.
    let priority = TaskPriority::new(10).unwrap();
    kernel
        .spawn_task(TaskId(1), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    // the first spawn itself triggers a switch -> see spawn_task's
    // preempt_current gate (it's what actually enters the first task).
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);

    kernel.on_tick_interrupt().unwrap();

    // TickAction::None -> no reschedule attempted, so no additional trigger.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
}

#[test]
fn on_tick_interrupt_does_not_trigger_switch_when_only_task_reselects_itself() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_SINGLE_TASK) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::RequestReschedule);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(10).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    kernel.on_tick_interrupt().unwrap();

    // the spawn itself triggers a switch (entering the first task); the
    // tick reselecting the same single task does not add another.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
}

#[test]
fn on_tick_interrupt_with_no_tasks_is_a_safe_no_op() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_NO_TASKS) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::RequestReschedule);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    let result = kernel.on_tick_interrupt();

    assert!(result.is_ok());
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
    assert_eq!(kernel.get_current_task(), None);
}

#[test]
fn spawn_task_propagates_ready_queue_full_error() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_QUEUE_FULL) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
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
    let pool = unsafe { &mut *addr_of_mut!(POOL_YIELD_SAME_PRIORITY) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    let priority = TaskPriority::new(12).unwrap();
    kernel
        .spawn_task(TaskId(1), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
    // the first spawn itself triggers a switch (entering the first task).
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);

    // first yield: advances straight to TaskId(2), no wasted self-reselect.
    kernel.yield_now().unwrap();
    assert_eq!(trigger_count.load(Ordering::SeqCst), 2);
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));

    // second yield: cycles back to TaskId(1).
    kernel.yield_now().unwrap();
    assert_eq!(trigger_count.load(Ordering::SeqCst), 3);
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
}

#[test]
fn on_tick_interrupt_acknowledges_tick_even_when_reschedule_errors() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_ACK_ON_ERROR) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, ack_count) = MockSystemTimer::new(TickAction::RequestReschedule);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    let priority = TaskPriority::new(9).unwrap();

    // first spawn becomes curr_task directly and is not enqueued.
    kernel
        .spawn_task(TaskId(0), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(0)));

    // fill this priority's ready queue to capacity (8) with other tasks, so
    // reschedule()'s attempt to requeue the current task on tick fails.
    for i in 1..=8u32 {
        kernel
            .spawn_task(TaskId(i), priority, 1024, dummy_entry, null_mut())
            .unwrap();
    }

    let result = kernel.on_tick_interrupt();

    assert_eq!(result, Err(KernelError::ReadyQueueFull));
    // acknowledge must still have run, despite reschedule() failing.
    assert_eq!(ack_count.load(Ordering::SeqCst), 1);
    // reschedule() errors out (on the full requeue) before ever reaching the
    // switched/context wiring, so only the initial spawn's trigger counts.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
}

#[test]
fn spawn_task_preempts_current_when_higher_priority() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_PREEMPT) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    // low-priority task spawns first and becomes curr_task.
    kernel
        .spawn_task(TaskId(1), TaskPriority::new(20).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
    // the first spawn itself triggers a switch (entering the first task).
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);

    // higher-priority task spawns second -> must preempt immediately, not
    // wait for the next yield/tick.
    kernel
        .spawn_task(TaskId(2), TaskPriority::new(5).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
    assert_eq!(trigger_count.load(Ordering::SeqCst), 2);

    // TaskId(2) remains the highest-priority ready task -> yielding does NOT
    // hand control back to the lower-priority TaskId(1). strict FPP priority
    // beats a cooperative yield; TaskId(1) only runs once TaskId(2) actually
    // blocks or completes (no such API exists yet in this phase).
    // yield_now() still unconditionally signals a switch even though the
    // selected task didn't change (matches its existing tested semantics).
    kernel.yield_now().unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
    assert_eq!(trigger_count.load(Ordering::SeqCst), 3);
}

#[test]
fn spawn_task_does_not_preempt_when_lower_priority() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_NO_PREEMPT) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(5).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), TaskPriority::new(20).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    assert_eq!(kernel.get_current_task(), Some(TaskId(1)));
    // the first spawn itself triggers a switch; the second, lower-priority
    // spawn does not preempt, so it adds no further trigger.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
}

#[test]
fn spawn_task_preemption_actually_requeues_previous_task() {
    // TaskId(0) and TaskId(2)'s two-task test can't distinguish "the
    // preempted task was correctly requeued" from "it was silently
    // dropped" -> both look identical when only one other task exists,
    // since the higher-priority task wins regardless. Prove the requeue
    // genuinely happens by filling the preempted task's own priority queue
    // to capacity first, so the requeue call inside spawn_task's preemption
    // path is forced to fail if (and only if) it actually runs.
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_PREEMPT_REQUEUE_FULL) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    let low_priority = TaskPriority::new(20).unwrap();

    // TaskId(0) becomes curr_task directly (nothing running yet).
    kernel
        .spawn_task(TaskId(0), low_priority, 1024, dummy_entry, null_mut())
        .unwrap();

    // fill TaskId(0)'s own priority queue to capacity (8) with unrelated
    // same-priority tasks -> none of these preempt TaskId(0) (same
    // priority never preempts), they just enqueue normally.
    for i in 1..=8u32 {
        kernel
            .spawn_task(TaskId(i), low_priority, 1024, dummy_entry, null_mut())
            .unwrap();
    }

    // a higher-priority spawn now preempts TaskId(0) -> its requeue attempt
    // hits the already-full queue and must fail. TaskId(9) is used (not a
    // larger id) since the test harness's fixed-size slot arrays only
    // support TaskId values below TEST_MAX_TASKS (32).
    let result = kernel.spawn_task(
        TaskId(9),
        TaskPriority::new(5).unwrap(),
        1024,
        dummy_entry,
        null_mut(),
    );

    assert_eq!(result, Err(KernelError::ReadyQueueFull));
}

#[test]
fn spawn_task_first_task_activates_context_with_no_outgoing() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_FIRST_TASK_CONTEXT) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let activated_contexts = mock_ctx_switch.activated_contexts.clone();
    let outgoing_contexts = mock_ctx_switch.outgoing_contexts.clone();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(1).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    // the first-ever switch has nothing to save -> outgoing side is None,
    // but the incoming side must still point PendSV at the new task.
    assert_eq!(*outgoing_contexts.lock().unwrap(), vec![None]);
    assert_eq!(activated_contexts.lock().unwrap().len(), 1);
}

#[test]
fn spawn_task_preemption_activates_correct_outgoing_and_incoming_context() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_PREEMPT_CONTEXT) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let activated_contexts = mock_ctx_switch.activated_contexts.clone();
    let outgoing_contexts = mock_ctx_switch.outgoing_contexts.clone();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    kernel
        .spawn_task(TaskId(1), TaskPriority::new(20).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), TaskPriority::new(5).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    let activated = activated_contexts.lock().unwrap().clone();
    let outgoing = outgoing_contexts.lock().unwrap().clone();

    // two switches: entering TaskId(1), then preempting it for TaskId(2).
    assert_eq!(activated.len(), 2);
    assert_eq!(outgoing, vec![None, Some(activated[0])]);
    // distinct tasks must not share a context value.
    assert_ne!(activated[0], activated[1]);
}

#[test]
fn on_tick_interrupt_switch_activates_correct_outgoing_and_incoming_context() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_TICK_SWITCH_CONTEXT) };
    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let activated_contexts = mock_ctx_switch.activated_contexts.clone();
    let outgoing_contexts = mock_ctx_switch.outgoing_contexts.clone();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::RequestReschedule);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        FppScheduler::new(),
        test_resources(pool),
        mock_timer,
    );

    // same priority -> the second spawn does not preempt; only the tick
    // below drives the actual switch, isolating it from spawn-time preemption.
    let priority = TaskPriority::new(10).unwrap();
    kernel
        .spawn_task(TaskId(1), priority, 1024, dummy_entry, null_mut())
        .unwrap();
    kernel
        .spawn_task(TaskId(2), priority, 1024, dummy_entry, null_mut())
        .unwrap();

    // only the first spawn (entering TaskId(1)) has activated a context so far.
    assert_eq!(activated_contexts.lock().unwrap().len(), 1);

    kernel.on_tick_interrupt().unwrap();

    let activated = activated_contexts.lock().unwrap().clone();
    let outgoing = outgoing_contexts.lock().unwrap().clone();

    // the tick-driven switch adds a second activation (TaskId(2)) and its
    // matching outgoing save of TaskId(1)'s context.
    assert_eq!(activated.len(), 2);
    assert_eq!(outgoing, vec![None, Some(activated[0])]);
}
