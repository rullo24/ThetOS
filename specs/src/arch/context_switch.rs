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

    /// point the next PendSV restore at this task's context
    fn activate_next_task(&self, ctx: &Self::TaskContext);

    /// point PendSV's save side at the outgoing task's context slot (None if nothing was running)
    fn set_current_task_context(&self, ctx: Option<*mut Self::TaskContext>);
}
