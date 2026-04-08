
// local imports
use specs::common::TaskId;
use specs::kernel::SchedulerPolicy;

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
    fn on_task_spawn(&mut self, task_id: TaskId) {
        // TODO: implement in phase 3
    }
}