
/// core imports
use core::result::Result;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackGuardMode {
    Canary,
    Watermark,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackGuardError {
    InvalidStackBounds,
    GuardCorrupted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StackGuardConfig {
    pub mode: StackGuardMode,
    pub canary_word: u32, // magic num written to stack boundary
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StackGuardState {
    pub low_mark: *mut u8, // base of the stack (limit for downward growth)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StackGuardContext {
    pub stack_top: *mut u8,
    pub stack_limit: *mut u8,
    pub state: StackGuardState,
    pub config: StackGuardConfig,
}

/// query stack protection for task stack guard
pub trait StackGuard {

    /// DESCRIPTION
    /// initialise the stack guard state -> returns Ok if initialised, Err if invalid
    fn initialise(
        &self,
        ctx: &mut StackGuardContext,
    ) -> Result<StackGuardState, StackGuardError>;

    /// DESCRIPTION
    /// check the stack guard state -> returns Ok if guard is intact, Err if corrupted
    fn check(
        &self,
        ctx: &mut StackGuardContext,
    ) -> Result<(), StackGuardError>;
}
