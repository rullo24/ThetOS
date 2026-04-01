#[repr(transparent)] // ensures the size of the struct is the same as the size of the fields
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct V7mTaskContext {
    pub sp: *mut u8, // stack pointer
}

impl V7mTaskContext {
    
    /// DESCRIPTION
    /// create a new task context
    pub fn new(sp: *mut u8) -> Self {
        Self { sp }
    }

}