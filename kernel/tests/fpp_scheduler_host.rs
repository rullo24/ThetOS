use kernel::FppScheduler;
use specs::common::TaskId;
use specs::kernel::{SchedulerPolicy, TaskPriority};

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