
use core::result::Result;

pub trait SystemTimer {
    type Error: core::fmt::Debug;

    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn setReload(&mut self, ticks: u32) -> Result<(), Self::Error>;
    fn currentTick(&self) -> Result<u64, Self::Error>;
    fn enableInterrupt(&mut self) -> Result<(), Self::Error>;
    fn disableInterrupt(&mut self) -> Result<(), Self::Error>;
    fn clearPending(&mut self) -> Result<(), Self::Error>;
}
