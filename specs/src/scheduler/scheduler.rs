
/// unique identifier for a task
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TaskId(pub u32);

/// kernel scheduler policy contract.
pub trait SchedulerPolicy {
    fn onTaskSpawn(&mut self, task_id: TaskId);
}