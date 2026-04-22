// local imports
use specs::kernel::{KernelError, TaskPriority};

#[test]
fn new_accepts_min_priority() {
    let priority = TaskPriority::new(TaskPriority::MIN).unwrap();
    assert_eq!(priority.as_u8(), TaskPriority::MIN);
}

#[test]
fn new_accepts_max_priority() {
    let priority = TaskPriority::new(TaskPriority::MAX).unwrap();
    assert_eq!(priority.as_u8(), TaskPriority::MAX);
}

#[test]
fn new_rejects_invalid_priority() {
    let result = TaskPriority::new(TaskPriority::MAX + 1);
    assert_eq!(result, Err(KernelError::InvalidPriority));
}

#[test]
fn default_is_within_supported_bounds() {
    let level = TaskPriority::default().as_u8();
    assert!(level >= TaskPriority::MIN);
    assert!(level <= TaskPriority::MAX);
}

#[test]
fn ordering_matches_fpp_intent() {
    let lower = TaskPriority::new(4).unwrap();
    let higher = TaskPriority::new(8).unwrap();
    assert!(higher > lower);
}