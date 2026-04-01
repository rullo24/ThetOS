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
    fn taskId(&self) -> TaskId;
    fn stackBounds(&self) -> StackBounds;
    fn context(&self) -> &Context;
    fn contextMut(&mut self) -> &mut Context;
    fn state(&self) -> TaskState;
    fn setState(&mut self, state: TaskState);
}

#[derive(Clone, Copy, Debug)]
pub struct TaskControlBlock<Context> {
    pub task_id: TaskId,
    pub stack_bounds: StackBounds,
    pub task_state: TaskState,
    pub task_context: Context,
}

impl<Context> CoreTcb<Context> for TaskControlBlock<Context> {
    fn taskId(&self) -> TaskId {
        self.task_id
    }

    fn stackBounds(&self) -> StackBounds {
        self.stack_bounds
    }

    fn context(&self) -> &Context {
        &self.task_context
    }

    fn contextMut(&mut self) -> &mut Context {
        &mut self.task_context
    }

    fn state(&self) -> TaskState {
        self.task_state
    }

    fn setState(&mut self, state: TaskState) {
        self.task_state = state;
    }
}
