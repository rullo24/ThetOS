// REFERENCE: https://medium.com/embedworld/arm-cortex-pendsv-the-core-mechanism-behind-rtos-task-switching-7679e68e68da

use crate::common::CORTEX_M_STACK_ALIGNMENT_BYTES;
use specs::arch::ContextSwitch;
use super::exception_frame::{v7m_default_task_exit, V7mHwExceptionFrame, V7mTaskInitialStackHead};
use super::V7mTaskContext; // pull in the task context type
use super::system_control_block::request_pendsv_pending;

pub struct V7mContextSwitch;

impl ContextSwitch for V7mContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize = CORTEX_M_STACK_ALIGNMENT_BYTES;
    type TaskContext = V7mTaskContext; // context for arch type

    /// DESCRIPTION
    /// initialise task context for the new task
    fn initialise_task_context(
        &self,
        stack_top: *mut u8,
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Self::TaskContext {
        let stack_addr = stack_top as usize;       

        // checking if stack_top is unaligned or null
        if stack_addr == 0 || (stack_addr % Self::STACK_ALIGNMENT_BYTES) != 0 {
            panic!("unaligned or null stack_top for v7m task context"); // cannot comptime check because stack_top is a pointer (runtime-changing)
        }

        // basic check (stack must large enough to hold the exception frame)
        if stack_addr < V7mTaskInitialStackHead::HEAD_SIZE_BYTES {
            panic!("stack_top too small for v7m task context");
        }

        let p_head_base = stack_top as usize - V7mTaskInitialStackHead::HEAD_SIZE_BYTES;
        if p_head_base == 0x0 {
            panic!("task initial stack head base is null");
        }
        if p_head_base % Self::STACK_ALIGNMENT_BYTES != 0 {
            panic!("task initial stack head base would leave misaligned SP");
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
            );
        }

        return V7mTaskContext::new(p_head_base as *mut u8);
    }

    /// DESCRIPTION
    /// trigger a PendSV switch to switch to the next task
    fn trigger_pendsv_switch(&self) {
        unsafe {
            request_pendsv_pending(); // set PendSV pending bit HIGH
        }
    }

}