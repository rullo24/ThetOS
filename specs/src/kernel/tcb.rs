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
