use core::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelError {
    InvalidState,
    Busy,
    InvalidConfig,
    Unsupported,
    StackGuard,
    InvalidPriority,
    ReadyQueueFull,
    TimerFault,
}

/// DESCRIPTION
/// implements Display method for printing error message from KernelError enum
impl Display for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            KernelError::InvalidState => f.write_str("invalid state"),
            KernelError::Busy => f.write_str("busy"),
            KernelError::InvalidConfig => f.write_str("invalid config"),
            KernelError::Unsupported => f.write_str("unsupported"),
            KernelError::StackGuard => f.write_str("stack guard violation"),
            KernelError::InvalidPriority => f.write_str("invalid priority"),
            KernelError::ReadyQueueFull => f.write_str("ready queue full"),
            KernelError::TimerFault => f.write_str("system timer fault"),
        }
    }
}

/// DESCRIPTION
/// alias for the result type
pub type Result<T> = core::result::Result<T, KernelError>;
