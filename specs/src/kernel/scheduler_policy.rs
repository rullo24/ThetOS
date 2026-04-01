use crate::common::TaskId;

/// kernel scheduler policy contract.
pub trait SchedulerPolicy {
    fn onTaskSpawn(&mut self, task_id: TaskId);
}
