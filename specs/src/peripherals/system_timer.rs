
use core::result::Result;

pub trait SystemTimer {
    type Error: core::fmt::Debug;

    fn setReload(&mut self, ticks: u32) -> Result<(), Self::Error>;
    fn clearPending(&mut self) -> Result<(), Self::Error>;

    // TODO: add more methods as required by hardware
}