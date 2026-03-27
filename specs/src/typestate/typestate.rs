
// local namespace for typestate State trait (can't define outside of this module)
mod sealed {
    pub trait Sealed {}
}

/// defining State trait + variants
pub trait State: sealed::Sealed{}
pub struct Uninitialised;
pub struct Enabled;

/// ensuring that the variants of the State trait cannot be changed outside of this module
impl sealed::Sealed for Uninitialised {}
impl sealed::Sealed for Enabled {}

/// register the variants of the State trait
impl State for Uninitialised {}
impl State for Enabled {}