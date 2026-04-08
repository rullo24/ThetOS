/// Shared Cortex-M constants used across variants.

pub const CORTEX_M_STACK_ALIGNMENT_BYTES: usize = 8;

pub mod interrupts;

pub use interrupts::{disable_interrupts, enable_interrupts, irqs_available_snapshot};