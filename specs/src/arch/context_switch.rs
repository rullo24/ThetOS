use super::error::ContextSwitchError;

/// used to store arch port -> Task Control Block (TCB)
pub trait ContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize;
    type TaskContext: Sized;

    fn initialise_task_context(
        &self,
        stack_top: *mut u8,
        stack_limit: *mut u8, // lowest valid addr for this stack region
        entry_point: extern "C" fn(*mut ()) -> !, // fixed calling ABI for task entry
        entry_arg: *mut (),
    ) -> Result<Self::TaskContext, ContextSwitchError>;

    fn trigger_pendsv_switch(&self);
}
