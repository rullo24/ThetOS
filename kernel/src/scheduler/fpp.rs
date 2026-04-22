
// local imports
use specs::kernel::{SchedulerPolicy, TaskPriority};
use specs::common::TaskId;

/// DESCRIPTION
/// fixed-priority preemptive scheduler implementation
pub struct FppScheduler;

impl FppScheduler {

    /// DESCRIPTION
    /// create a new fixed-priority preemptive scheduler instance
    pub const fn new() -> Self {
        Self
    }
}

impl SchedulerPolicy for FppScheduler {
    
    /// DESCRIPTION
    /// register task spawn into scheduler state
    fn register_task(&mut self, _task_id: TaskId, _priority: TaskPriority) {
        // TODO: implement in phase 3
    }

    /// DESCRIPTION
    /// mark task runnable and place it in policy-managed ready structures.
    fn enqueue_runnable(&mut self, _task_id: TaskId, _priority: TaskPriority) {
        // TODO: implement in phase 3
    }

    /// DESCRIPTION
    /// choose next runnable task according to policy ordering.
    fn select_next_runnable(&mut self) -> Option<TaskId> {
        // TODO: implement in phase 3
        return None;
    }

    /// DESCRIPTION
    /// decide whether candidate should preempt current under policy rules.
    fn should_preempt_current(&self, _current: Option<(TaskId, TaskPriority)>, _candidate: (TaskId, TaskPriority)) -> bool {
        // TODO: implement in phase 3
        return false;
    }

}