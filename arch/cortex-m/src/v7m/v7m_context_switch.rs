// REFERENCE: https://medium.com/embedworld/arm-cortex-pendsv-the-core-mechanism-behind-rtos-task-switching-7679e68e68da

use crate::common::CORTEX_M_STACK_ALIGNMENT_BYTES;
use specs::arch::ContextSwitch;
use super::v7m_exception_frame::{v7m_default_task_exit, V7mHwExceptionFrame, V7mTaskInitialStackHead};
use super::V7mTaskContext; // pull in the task context type
use super::v7m_system_control_block::request_pendsv_pending;
use super::{set_current_task_tcb, set_next_task_psp};
use specs::arch::error::ContextSwitchError;

pub struct V7mContextSwitch;

impl ContextSwitch for V7mContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize = CORTEX_M_STACK_ALIGNMENT_BYTES;
    type TaskContext = V7mTaskContext; // context for arch type

    /// DESCRIPTION
    /// initialise task context for the new task
    fn initialise_task_context(
        &self,
        stack_top: *mut u8, // highest addr valid for this stack
        stack_limit: *mut u8, // lowest addr valid for this stack
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Result<Self::TaskContext, ContextSwitchError> {
        let stack_top_u = stack_top as usize;       
        let limit_u = stack_limit as usize;

        if stack_top_u == 0 || limit_u == 0 {
            return Err(ContextSwitchError::NullStackPointer);
        }

        if stack_top_u <= limit_u {
            return Err(ContextSwitchError::InvalidStackBounds);
        }

        if (stack_top_u % Self::STACK_ALIGNMENT_BYTES) != 0 {
            return Err(ContextSwitchError::UnalignedStackTop);
        }

        if stack_top_u - limit_u < V7mTaskInitialStackHead::HEAD_SIZE_BYTES {
            return Err(ContextSwitchError::StackRegionTooSmall);
        }

        let Some(p_head_base) = stack_top_u.checked_sub(V7mTaskInitialStackHead::HEAD_SIZE_BYTES) else {
            return Err(ContextSwitchError::StackRegionTooSmall);
        };

        if p_head_base < limit_u {
            return Err(ContextSwitchError::InvalidStackBounds);
        }

        if (p_head_base % Self::STACK_ALIGNMENT_BYTES) != 0 {
            return Err(ContextSwitchError::UnalignedStackTop);
        }

        // write the initial task frame registers into the exception frame
        let task_exit_lr: u32 = v7m_default_task_exit as *const () as usize as u32; // func ptr made to u32 addr -> thumb tracking bit set in exception frame initialisation
        unsafe {
                       
            // write the initial task frame registers into the callee-saved frame
            core::ptr::write_bytes(
                p_head_base as *mut u8,
                0x0,
                V7mTaskInitialStackHead::CALLEE_SIZE_BYTES,
            );

            // write the initial task frame registers into the hardware frame
            V7mHwExceptionFrame::write_initial_task_frame(
                V7mTaskInitialStackHead::get_hw_frame_ptr(p_head_base as *mut u8),
                entry_point,
                entry_arg,
                task_exit_lr,
            )?; // send errors upwards if fails
        }

        Ok(V7mTaskContext::new(p_head_base as *mut u8)) // return valid context
    }

    /// DESCRIPTION
    /// trigger a PendSV switch to switch to the next task
    fn trigger_yield(&self) {
        unsafe {
            request_pendsv_pending(); // set PendSV pending bit HIGH
        }
    }

    /// DESCRIPTION
    /// point the next PendSV restore at this task's context
    fn activate_next_task(&self, ctx: &Self::TaskContext) {
        unsafe {
            set_next_task_psp(ctx.sp);
        }
    }

    /// DESCRIPTION
    /// point PendSV's save side at the outgoing task's context slot (null skips the save)
    fn set_current_task_context(&self, ctx: Option<*mut Self::TaskContext>) {
        unsafe {
            set_current_task_tcb(ctx.unwrap_or(core::ptr::null_mut()));
        }
    }

}