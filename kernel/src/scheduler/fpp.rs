// external imports
use heapless::Deque;

// local imports
use specs::common::TaskId;
use specs::kernel::{
    KernelError,
    Result,
    SchedulerPolicy, 
    TaskPriority,
};

// constants
const PRIORITY_LEVELS: usize = TaskPriority::TASK_LEVELS; // check priority range -> not matching specs/ will cause silent bugs or UB
const _: () = assert!(TaskPriority::MAX >= TaskPriority::MIN);
const _: () = assert!(PRIORITY_LEVELS > 0);
const _: () = assert!(PRIORITY_LEVELS == (TaskPriority::MAX as usize) - (TaskPriority::MIN as usize) + 1);
const READY_QUEUE_CAPACITY: usize = 8; // arbitrary capacity for ready queue

/// DESCRIPTION
/// fixed-priority preemptive scheduler implementation
pub struct FppScheduler {
    ready_queues: [Deque<TaskId, READY_QUEUE_CAPACITY>; PRIORITY_LEVELS], // ready queues for each priority level
}

impl FppScheduler {

    /// DESCRIPTION
    /// create a new fixed-priority preemptive scheduler instance
    pub const fn new() -> Self {
        const EMPTY: Deque<TaskId, READY_QUEUE_CAPACITY> = Deque::new();
        Self {
            ready_queues: [EMPTY; PRIORITY_LEVELS], // init all ready queues for priorities
        }
    }

    /// DESCRIPTION
    /// convert priority to index for ready queue array
    fn priority_index(priority: TaskPriority) -> usize {
        (priority.as_u8() - TaskPriority::MIN) as usize
    }

    /// DESCRIPTION
    /// enqueue task into ready queue for given priority
    fn enqueue_internal(&mut self, task_id: TaskId, priority: TaskPriority) -> Result<()>{
        let idx = Self::priority_index(priority);
        let queue = &mut self.ready_queues[idx];
        queue
            .push_back(task_id)
            .map_err(|_| KernelError::ReadyQueueFull)?; // map heapless err -> ThetOS err
        Ok(())
    }

    /// DESCRIPTION
    /// dequeue task from highest priority ready queue
    fn dequeue_highest_internal(&mut self) -> Option<TaskId> {
        // starting at highest priority (0) -> lowest (PRIORITY_LEVELS - 1)
        for idx in 0..PRIORITY_LEVELS {
            
            // if avail in current priority queue -> return task id
            if let Some(task_id) = self.ready_queues[idx].pop_front() {
                return Some(task_id);
            }

        }
        return None;
    }

}

impl SchedulerPolicy for FppScheduler {
    
    /// DESCRIPTION
    /// register task spawn into scheduler state
    fn register_task(&mut self, task_id: TaskId, priority: TaskPriority) -> Result<()> {
        self.enqueue_internal(task_id, priority)
    }

    /// DESCRIPTION
    /// mark task runnable and place it in policy-managed ready structures.
    fn enqueue_runnable(&mut self, task_id: TaskId, priority: TaskPriority) -> Result<()> {
        self.enqueue_internal(task_id, priority)
    }

    /// DESCRIPTION
    /// choose next runnable task according to policy ordering.
    fn select_next_runnable(&mut self) -> Option<TaskId> {
        return self.dequeue_highest_internal();
    }

    /// DESCRIPTION
    /// decide whether candidate should preempt current under policy rules.
    fn should_preempt_current(&self, _current: Option<(TaskId, TaskPriority)>, _candidate: (TaskId, TaskPriority)) -> bool {
        return false; // TODO: to be implemented in phase 3
    }

}