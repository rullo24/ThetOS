use crate::common::CORTEX_M_STACK_ALIGNMENT_BYTES;
use specs::arch::ContextSwitch;

pub struct V7mContextSwitch;

impl ContextSwitch for V7mContextSwitch {
    const STACK_ALIGNMENT_BYTES: usize = CORTEX_M_STACK_ALIGNMENT_BYTES;
    type TaskContext = *mut u8; // TODO: checkout this

    fn initialise_task_context(
    ) ->Self:TaskContext {
        panic!("initialise_task_context not implemented");

        // TODO: implement initialise_task_context

    }

    fn trigger_pend_switch(&self) {
        panic!("trigger_pend_switch not implemented");

        // TODO: implement trigger_pend_switch

    }

}