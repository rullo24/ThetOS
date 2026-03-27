
/// used to store arch port -> Task Control Block (TCB)
pub trait ContextSwitch {
    type TaskContext: Sized;
    fn initialiseTaskContext(
        &self,
        stack_top: *mut u8,
        entry_point: extern "C" fn(*mut ()),
        entry_arg: *mut (),
    ) -> Self::TaskContext;
    fn triggerPendSwitch(&self);
}