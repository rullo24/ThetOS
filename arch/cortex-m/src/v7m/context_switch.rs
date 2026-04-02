use crate::common::CORTEX_M_STACK_ALIGNMENT_BYTES;
use specs::arch::ContextSwitch;
use super::V7mTaskContext; // pull in the task context type

pub struct V7mContextSwitch;

// REFERENCE: arm developer -> Home / Documentation / IP Products / Processors / Cortex-M / Cortex-M3 / Cortex-M3 Devices Generic User Guide / Exception entry and return
// https://developer.arm.com/documentation/dui0552/a/the-cortex-m3-processor/exception-model/exception-entry-and-return?lang=en
// exception entry frame: R0-R3, R12, LR, PC, xPSR (ARMv7-M -> no FPU)
const V7M_NUM_BASIC_EXCEPTION_FRAME_REGISTERS: usize = 8;
const V7M_BASIC_EXCEPTION_FRAME_BYTES: usize = V7M_NUM_BASIC_EXCEPTION_FRAME_REGISTERS * core::mem::size_of::<u32>();

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
    ) -> Self::TaskContext {
        let stack_addr = stack_top as usize;       
        let align = Self::STACK_ALIGNMENT_BYTES;

        // checking if stack_top is unaligned or null
        if stack_addr == 0 || (stack_addr % align) != 0 {
            panic!("unaligned or null stack_top for v7m task context"); // cannot comptime check because stack_top is a pointer (runtime-changing)
        }

        return V7mTaskContext::new(stack_top);
    }

    /// DESCRIPTION
    /// trigger a pend switch to switch to the next task
    fn trigger_pend_switch(&self) {
        panic!("trigger_pend_switch not implemented");
    }

}