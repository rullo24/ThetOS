/// Shared Cortex-M constants used across variants.

pub const CORTEX_M_STACK_ALIGNMENT_BYTES: usize = 8;

pub mod interrupts;
pub mod critical_section;
pub mod stack_guard;
pub mod wfi;

pub use interrupts::{
    disable_interrupts,
    enable_interrupts,
    irqs_available_snapshot,
    irsq_available_snapshot_and_disable,
    set_irqs_primask,
};
pub use critical_section::CortexMCriticalSection;
pub use stack_guard::CortexMStackGuard;
pub use wfi::wfi;