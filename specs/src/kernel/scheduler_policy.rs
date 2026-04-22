// local imports
use crate::common::TaskId;
use crate::kernel::{
    Result,
    TaskPriority,
};

/// kernel scheduler policy contract.
pub trait SchedulerPolicy {
    /// DESCRIPTION
    /// register a new task with its base priority.
    fn register_task(&mut self, task_id: TaskId, priority: TaskPriority) -> Result<()>;

    /// DESCRIPTION
    /// mark task runnable and place it in policy-managed ready structures.
    fn enqueue_runnable(&mut self, task_id: TaskId, priority: TaskPriority) -> Result<()>;

    /// DESCRIPTION
    /// choose next runnable task according to policy ordering.
    fn select_next_runnable(&mut self) -> Option<TaskId>;

    /// DESCRIPTION
    /// decide whether candidate should preempt current under policy rules.
    fn should_preempt_current(
        &self,
        current: Option<(TaskId, TaskPriority)>,
        candidate: (TaskId, TaskPriority),
    ) -> bool;

}
