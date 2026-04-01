use specs::common::TaskId;
use specs::kernel::{CoreTcb, StackBounds, TaskState};

#[derive(Clone, Copy, Debug)]
pub struct TaskControlBlock<Context> {
    pub task_id: TaskId,
    pub stack_bounds: StackBounds,
    pub task_state: TaskState,
    pub task_context: Context,
}

impl<Context> CoreTcb<Context> for TaskControlBlock<Context> {
    /// DESCRIPTION
    /// returns task identifier.
    fn get_task_id(&self) -> TaskId {
        self.task_id
    }

    /// DESCRIPTION
    /// returns stack boundary metadata.
    fn get_stack_bounds(&self) -> StackBounds {
        self.stack_bounds
    }

    /// DESCRIPTION
    /// returns immutable context reference.
    fn get_context(&self) -> &Context {
        &self.task_context
    }

    /// DESCRIPTION
    /// returns mutable context reference.
    fn get_context_mut(&mut self) -> &mut Context {
        &mut self.task_context
    }

    /// DESCRIPTION
    /// returns current task state.
    fn get_state(&self) -> TaskState {
        self.task_state
    }

    /// DESCRIPTION
    /// updates current task state.
    fn set_state(&mut self, state: TaskState) {
        self.task_state = state;
    }
}
