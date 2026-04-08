
// local imports
use specs::kernel::SchedulerPolicy;
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
    fn on_task_spawn(&mut self, _task_id: TaskId) {
        // TODO: implement in phase 3
    }
}