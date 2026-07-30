use kernel::FppScheduler;
use specs::common::TaskId;
use specs::kernel::{KernelError, SchedulerPolicy, TaskPriority};

#[test]
fn selects_highest_priority_first() {
    let mut scheduler = FppScheduler::new();

    assert!(scheduler
        .enqueue_runnable(TaskId(1), TaskPriority::new(10).unwrap())
        .is_ok());
    assert!(scheduler
        .enqueue_runnable(TaskId(2), TaskPriority::new(1).unwrap())
        .is_ok());
    assert!(scheduler
        .enqueue_runnable(TaskId(3), TaskPriority::new(20).unwrap())
        .is_ok());

    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(2)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(1)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(3)));
    assert_eq!(scheduler.select_next_runnable(), None);
}

#[test]
fn preserves_fifo_within_same_priority() {
    let mut scheduler = FppScheduler::new();
    let priority = TaskPriority::new(5).unwrap();

    assert!(scheduler.enqueue_runnable(TaskId(10), priority).is_ok());
    assert!(scheduler.enqueue_runnable(TaskId(11), priority).is_ok());
    assert!(scheduler.enqueue_runnable(TaskId(12), priority).is_ok());

    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(10)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(11)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(12)));
    assert_eq!(scheduler.select_next_runnable(), None);
}

#[test]
fn preempts_when_no_current_task() {
    let scheduler = FppScheduler::new();
    let candidate = (TaskId(100), TaskPriority::new(3).unwrap());

    assert!(scheduler.should_preempt_current(None, candidate));
}

#[test]
fn preempts_when_candidate_has_higher_priority() {
    let scheduler = FppScheduler::new();
    let current = Some((TaskId(1), TaskPriority::new(10).unwrap()));
    let candidate = (TaskId(2), TaskPriority::new(5).unwrap());

    assert!(scheduler.should_preempt_current(current, candidate));
}

#[test]
fn does_not_preempt_when_candidate_has_lower_priority() {
    let scheduler = FppScheduler::new();
    let current = Some((TaskId(1), TaskPriority::new(2).unwrap()));
    let candidate = (TaskId(2), TaskPriority::new(10).unwrap());

    assert!(!scheduler.should_preempt_current(current, candidate));
}

#[test]
fn does_not_preempt_when_same_priority() {
    let scheduler = FppScheduler::new();
    let current = Some((TaskId(1), TaskPriority::new(5).unwrap()));
    let candidate = (TaskId(2), TaskPriority::new(5).unwrap());

    assert!(!scheduler.should_preempt_current(current, candidate));
}

#[test]
fn ready_queue_accepts_new_task_after_dequeue_at_capacity() {
    let mut scheduler = FppScheduler::new();
    let priority = TaskPriority::new(7).unwrap();

    // fill the priority's ready queue to capacity (8).
    for i in 0..8u32 {
        assert!(scheduler.enqueue_runnable(TaskId(i), priority).is_ok());
    }

    // queue is full -> next enqueue must fail.
    assert_eq!(
        scheduler.enqueue_runnable(TaskId(100), priority),
        Err(KernelError::ReadyQueueFull)
    );

    // dequeue one -> frees a slot.
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(0)));

    // now a new enqueue at the same priority should succeed.
    assert!(scheduler.enqueue_runnable(TaskId(200), priority).is_ok());

    // remaining order: TaskId(1)..TaskId(7), then TaskId(200).
    for i in 1..8u32 {
        assert_eq!(scheduler.select_next_runnable(), Some(TaskId(i)));
    }
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(200)));
    assert_eq!(scheduler.select_next_runnable(), None);
}

#[test]
fn draining_one_priority_queue_does_not_affect_another() {
    let mut scheduler = FppScheduler::new();
    let high_priority = TaskPriority::new(2).unwrap();
    let low_priority = TaskPriority::new(9).unwrap();

    assert!(scheduler.enqueue_runnable(TaskId(1), high_priority).is_ok());
    assert!(scheduler.enqueue_runnable(TaskId(2), high_priority).is_ok());
    assert!(scheduler.enqueue_runnable(TaskId(3), low_priority).is_ok());

    // drain the high-priority queue completely.
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(1)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(2)));

    // high-priority queue is now empty -> selection must fall through to
    // the low-priority queue, not return None or something stale.
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(3)));
    assert_eq!(scheduler.select_next_runnable(), None);

    // re-adding to the now-empty high-priority queue still works and is
    // still selected ahead of anything at the lower priority.
    assert!(scheduler.enqueue_runnable(TaskId(4), high_priority).is_ok());
    assert!(scheduler.enqueue_runnable(TaskId(5), low_priority).is_ok());
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(4)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(5)));
}

#[test]
fn register_task_makes_task_selectable() {
    let mut scheduler = FppScheduler::new();
    let priority = TaskPriority::new(6).unwrap();

    assert!(scheduler.register_task(TaskId(1), priority).is_ok());
    assert!(scheduler.register_task(TaskId(2), priority).is_ok());

    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(1)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(2)));
    assert_eq!(scheduler.select_next_runnable(), None);
}

#[test]
fn enqueue_runnable_rejects_beyond_capacity() {
    let mut scheduler = FppScheduler::new();
    let priority = TaskPriority::new(11).unwrap();

    for i in 0..8u32 {
        assert!(scheduler.enqueue_runnable(TaskId(i), priority).is_ok());
    }

    assert_eq!(
        scheduler.enqueue_runnable(TaskId(8), priority),
        Err(KernelError::ReadyQueueFull)
    );
}

#[test]
fn selects_across_priority_extremes() {
    let mut scheduler = FppScheduler::new();
    let min_priority = TaskPriority::new(TaskPriority::MIN).unwrap();
    let max_priority = TaskPriority::new(TaskPriority::MAX).unwrap();

    assert!(scheduler.enqueue_runnable(TaskId(1), max_priority).is_ok());
    assert!(scheduler.enqueue_runnable(TaskId(2), min_priority).is_ok());

    // MIN priority value (0) is the highest priority -> selected first.
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(2)));
    assert_eq!(scheduler.select_next_runnable(), Some(TaskId(1)));
    assert_eq!(scheduler.select_next_runnable(), None);
}