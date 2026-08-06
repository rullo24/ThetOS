use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::Ordering;
use kernel::scheduler::FppScheduler;
use kernel::Kernel;
use specs::common::TaskId;
use specs::kernel::{KernelError, TaskPriority, TickAction};

mod support;

use support::{
    dummy_entry, test_resources, MockContextSwitch, MockCriticalSection, MockScheduler,
    POOL_CRIT, POOL_KERNEL_INIT, POOL_KERNEL_START_NO_TASK, POOL_QUEUE_FULL,
    POOL_SPAWN_EXACT_FIT, POOL_SPAWN_FIRST_TASK_CONTEXT, POOL_SPAWN_NO_PREEMPT, POOL_SPAWN_OK,
    POOL_SPAWN_PREEMPT, POOL_SPAWN_PREEMPT_CONTEXT, POOL_SPAWN_PREEMPT_REQUEUE_FULL,
    POOL_SPAWN_REJECT, POOL_TICK_ACK_ON_ERROR, POOL_TICK_NO_ACTION, POOL_TICK_NO_TASKS,
    POOL_TICK_SINGLE_TASK, POOL_TICK_SWITCH, POOL_TICK_SWITCH_CONTEXT, POOL_YIELD,
    POOL_YIELD_SAME_PRIORITY,
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
fn spawn_task_stacks_exactly_fill_the_pool_with_no_gaps_or_overflow() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_EXACT_FIT) };
    let pool_base = pool.as_ptr() as usize;
    let pool_end = pool_base + pool.len();

    let (mock_ctx_switch, _trigger_count) = MockContextSwitch::new();
    let bounds = mock_ctx_switch.initialised_bounds.clone();
    let (mock_timer, _ack_count) = MockSystemTimer::new(TickAction::None);
    let mut kernel = Kernel::new(
        mock_ctx_switch,
        MockCriticalSection,
        MockScheduler,
        test_resources(pool),
        mock_timer,
    );

    // 4 tasks x 1024 bytes == the pool's exact size -> must fit with zero slack
    for id in 1..=4 {
        let result = kernel.spawn_task(
            TaskId(id),
            TaskPriority::new(1).unwrap(),
            1024,
            dummy_entry,
            null_mut(),
        );
        assert!(result.is_ok(), "task {id} failed to spawn within an exactly-sized pool");
    }
    assert_eq!(kernel.get_task_count(), 4);

    let bounds = bounds.lock().unwrap();
    assert_eq!(bounds.len(), 4);

    // contiguous, no gaps: first task starts exactly at the pool base, each next task's
    // stack_limit picks up exactly where the previous task's stack_top left off, and the
    // last task's stack_top lands exactly on the pool's end address -> not under, not over.
    assert_eq!(bounds[0].0, pool_base);
    for i in 1..bounds.len() {
        assert_eq!(bounds[i].0, bounds[i - 1].1, "gap or overlap between task {} and task {}", i, i + 1);
    }
    assert_eq!(bounds[bounds.len() - 1].1, pool_end);

    // one byte over the exact-fit boundary must be rejected, not silently overflowed
    let overflow_result = kernel.spawn_task(
        TaskId(5),
        TaskPriority::new(1).unwrap(),
        512, // arch alignment rounds this up, but the pool has 0 bytes left regardless
        dummy_entry,
        null_mut(),
    );
    assert_eq!(overflow_result, Err(KernelError::InvalidConfig));
    assert_eq!(kernel.get_task_count(), 4);
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
    // spawn_task is bookkeeping only -> no trigger until the tick below.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);

    kernel.on_tick_interrupt().unwrap();

    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
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
    // spawn_task is bookkeeping only -> no trigger from either spawn.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);

    kernel.on_tick_interrupt().unwrap();

    // TickAction::None -> no reschedule attempted, so no trigger at all.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
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

    // spawn_task never triggers, and reselecting the same single task
    // doesn't count as a switch either -> no trigger at all.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
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
    // spawn_task is bookkeeping only -> no trigger until the yields below.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);

    // first yield: advances straight to TaskId(2), no wasted self-reselect.
    kernel.yield_now().unwrap();
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));

    // second yield: cycles back to TaskId(1).
    kernel.yield_now().unwrap();
    assert_eq!(trigger_count.load(Ordering::SeqCst), 2);
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
    // spawn_task never triggers, and reschedule() errors out before it
    // could trigger one either -> no trigger at all.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
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
    // spawn_task is bookkeeping only -> no hardware trigger from a spawn.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);

    // higher-priority task spawns second -> preempts in the scheduler's
    // bookkeeping immediately (curr_task updates now), but the hardware
    // switch itself still only happens at the next reschedule.
    kernel
        .spawn_task(TaskId(2), TaskPriority::new(5).unwrap(), 1024, dummy_entry, null_mut())
        .unwrap();

    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);

    // TaskId(2) remains the highest-priority ready task -> yielding does NOT
    // hand control back to the lower-priority TaskId(1). strict FPP priority
    // beats a cooperative yield; TaskId(1) only runs once TaskId(2) actually
    // blocks or completes (no such API exists yet in this phase).
    // yield_now() still unconditionally triggers a switch even though the
    // selected task didn't change (matches its existing tested semantics) ->
    // this is the first and only trigger in this test.
    kernel.yield_now().unwrap();
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
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
    // spawn_task never triggers, regardless of preemption.
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
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
fn spawn_task_never_touches_yield_context_even_on_preemption() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_PREEMPT_CONTEXT) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
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

    // spawn_task is pure bookkeeping -> curr_task updates via preemption,
    // but neither the yield context hooks nor the trigger are ever touched.
    assert_eq!(kernel.get_current_task(), Some(TaskId(2)));
    assert_eq!(activated_contexts.lock().unwrap().len(), 0);
    assert_eq!(outgoing_contexts.lock().unwrap().len(), 0);
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
}

#[test]
fn kernel_start_dispatches_into_first_spawned_task() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_SPAWN_FIRST_TASK_CONTEXT) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
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
    // spawning alone never dispatches anything.
    assert_eq!(activated_contexts.lock().unwrap().len(), 0);
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);

    kernel.start().unwrap();

    // start() performs the one deferred dispatch: nothing was running
    // before (outgoing is None), and the spawned task is activated + triggered.
    assert_eq!(*outgoing_contexts.lock().unwrap(), vec![None]);
    assert_eq!(activated_contexts.lock().unwrap().len(), 1);
    assert_eq!(trigger_count.load(Ordering::SeqCst), 1);
}

#[test]
fn kernel_start_is_a_no_op_when_no_task_spawned() {
    let pool = unsafe { &mut *addr_of_mut!(POOL_KERNEL_START_NO_TASK) };
    let (mock_ctx_switch, trigger_count) = MockContextSwitch::new();
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

    let result = kernel.start();

    assert!(result.is_ok());
    assert_eq!(activated_contexts.lock().unwrap().len(), 0);
    assert_eq!(outgoing_contexts.lock().unwrap().len(), 0);
    assert_eq!(trigger_count.load(Ordering::SeqCst), 0);
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

    // spawn_task never activates anything on its own.
    assert_eq!(activated_contexts.lock().unwrap().len(), 0);

    kernel.on_tick_interrupt().unwrap();

    let activated = activated_contexts.lock().unwrap().clone();
    let outgoing = outgoing_contexts.lock().unwrap().clone();

    // the tick-driven switch is the only dispatch in this test: TaskId(2)
    // activated, with TaskId(1)'s context correctly captured as outgoing.
    assert_eq!(activated.len(), 1);
    assert_eq!(outgoing.len(), 1);
    assert!(outgoing[0].is_some());
    // distinct tasks must not share a context value.
    assert_ne!(outgoing[0], Some(activated[0]));
}
