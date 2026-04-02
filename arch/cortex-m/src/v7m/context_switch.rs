use crate::common::CORTEX_M_STACK_ALIGNMENT_BYTES;
use specs::arch::ContextSwitch;
use super::exception_frame::{v7m_default_task_exit, V7mBasicExceptionFrame};
use super::V7mTaskContext; // pull in the task context type

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
        if stack_addr < V7mBasicExceptionFrame::FRAME_SIZE_BYTES {
            panic!("stack_top too small for v7m task context");
        }

        // calc stack pointer after frame is written
        let new_sp = stack_top as usize - V7mBasicExceptionFrame::FRAME_SIZE_BYTES; // -32 bytes for the exception frame
        if new_sp == 0x0 {
            panic!("frame base would be null");
        }
        if (new_sp % Self::STACK_ALIGNMENT_BYTES) != 0 {
            panic!("frame base would leave misaligned stack pointer");
        }

        // write the initial task frame registers into the exception frame
        let task_exit_lr: u32 = v7m_default_task_exit as *const () as usize as u32; // func ptr made to u32 addr -> thumb tracking bit set in exception frame initialisation
        unsafe {
            V7mBasicExceptionFrame::write_initial_task_frame(
                new_sp as *mut u8,
                entry_point,
                entry_arg,
                task_exit_lr,
            );
        }

        return V7mTaskContext::new(new_sp as *mut u8);
    }

    /// DESCRIPTION
    /// trigger a pend switch to switch to the next task
    fn trigger_pend_switch(&self) {
        panic!("trigger_pend_switch not implemented");
    }

}