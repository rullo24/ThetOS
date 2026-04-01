
/// used to store arch port -> Task Control Block (TCB)
pub trait ContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize;
    type TaskContext: Sized;

    fn initialise_task_context(
        &self,
        stack_top: *mut u8,
        entry_point: extern "C" fn(*mut ()) -> !, // fixed calling ABI for task entry
        entry_arg: *mut (),
    ) -> Self::TaskContext;

    fn trigger_pend_switch(&self);
}
