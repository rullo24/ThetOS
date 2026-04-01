
use core::result::Result;

/// query stack protection for task stack guard
pub trait StackGuard {
    type Error: core::fmt::Debug;

    fn install(&mut self, stack_bottom: *const u8, stack_len: usize) -> Result<(), Self::Error>;
}
