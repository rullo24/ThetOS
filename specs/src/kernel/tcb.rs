use crate::common::TaskId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StackBounds {
    pub bottom: *mut u8,
    pub top: *mut u8,
}

pub trait CoreTcb<Context> {
    fn task_id(&self) -> TaskId;
    fn stack_bounds(&self) -> StackBounds;
    fn context(&self) -> &Context;
    fn context_mut(&mut self) -> &mut Context;
    fn state(&self) -> TaskState;
    fn set_state(&mut self, state: TaskState);
}

#[derive(Clone, Copy, Debug)]
pub struct TaskControlBlock<Context> {
    pub task_id: TaskId,
    pub stack_bounds: StackBounds,
    pub task_state: TaskState,
    pub task_context: Context,
}

impl<Context> CoreTcb<Context> for TaskControlBlock<Context> {
    fn task_id(&self) -> TaskId {
        self.task_id
    }

    fn stack_bounds(&self) -> StackBounds {
        self.stack_bounds
    }

    fn context(&self) -> &Context {
        &self.task_context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.task_context
    }

    fn state(&self) -> TaskState {
        self.task_state
    }

    fn set_state(&mut self, state: TaskState) {
        self.task_state = state;
    }
}
