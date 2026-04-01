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
    fn get_task_id(&self) -> TaskId;
    fn get_stack_bounds(&self) -> StackBounds;
    fn get_context(&self) -> &Context;
    fn get_context_mut(&mut self) -> &mut Context;
    fn get_state(&self) -> TaskState;
    fn set_state(&mut self, state: TaskState);
}
