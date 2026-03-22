use core::fmt::{Display, Formatter, Result as FmtResult};

pub enum ThetosError {
    InvalidState,
    Busy,
    InvalidConfig,
    Unsupported,
} 

/// implement Display to print the error message
impl Display for ThetosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ThetosError::InvalidState => f.write_str("invalid state"),
            ThetosError::Busy => f.write_str("busy"),
            ThetosError::InvalidConfig => f.write_str("invalid config"),
            ThetosError::Unsupported => f.write_str("unsupported"),
        }
    }
}

/// alias for the result type
pub type Result<T> = core::result::Result<T, ThetosError>;