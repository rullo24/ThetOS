use crate::common::TaskId;

/// kernel scheduler policy contract.
pub trait SchedulerPolicy {
    fn on_task_spawn(&mut self, task_id: TaskId);
}
