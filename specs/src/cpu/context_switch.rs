
/// used to store arch port -> Task Control Block (TCB)
pub trait ContextSwitch {
    type TaskContext: Sized;
}