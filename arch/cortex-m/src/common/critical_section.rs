use crate::common::{irsq_available_snapshot_and_disable, set_irqs_primask};
use specs::kernel::CriticalSection;

pub struct CortexMCriticalSection;

impl CriticalSection for CortexMCriticalSection {
    fn with_execute<Res, Op>(&self, operation: Op) -> Res
    where Op: FnOnce() -> Res,
    {
        // snapshot and disable interrupts
        let initial_irq_state = irsq_available_snapshot_and_disable();

        // execute operation
        let result = operation();

        // restore interrupts to previous state
        set_irqs_primask(initial_irq_state);

        result // return operation result
    }
}