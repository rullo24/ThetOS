use crate::common::CORTEX_M_STACK_ALIGNMENT_BYTES;
use specs::arch::ContextSwitch;

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
        _entry_point: extern "C" fn(*mut ()) -> !,
        _entry_arg: *mut (),
    ) ->Self::TaskContext {
        let stack_addr = stack_top as usize;       
        let align = Self::STACK_ALIGNMENT_BYTES;

        if stack_addr == 0 || (stack_addr % align) != 0 {
            panic!("unaligned or null stack_top for v7m task context"); // cannot comptime check because stack_top is a pointer (runtime-changing)
        }

        V7mTaskContext::new(stack_top)
    }

    /// DESCRIPTION
    /// trigger a pend switch to switch to the next task
    fn trigger_pend_switch(&self) {
        panic!("trigger_pend_switch not implemented");
    }

}