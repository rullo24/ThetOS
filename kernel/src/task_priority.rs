use crate::kernel::{KernelError, Result};

/// fixed-priority used by scheduler
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TaskPriority(u8);

impl TaskPriority {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 31;
    pub const DEFAULT: Self = Self(15); // middle priority

    /// DESCRIPTION
    /// creates a priority if the value is within supported bounds
    pub const fn new(level: u8) -> Result<Self> {
        if level < Self::MIN || level > Self::MAX {
            Err(KernelError::InvalidPriority)
        }
        Ok(Self(level))
    }

    /// DESCRIPTION
    /// returns the priority as a u8
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl Default for TaskPriority {
    
    /// DESCRIPTION
    /// returns the default priority
    fn default() -> Self {
        Self::DEFAULT
    }

}