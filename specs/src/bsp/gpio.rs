use core::result::Result;

/// board-facing GPIO contract exposed to application code.
pub trait GpioPin {
    type Error: core::fmt::Debug;

    fn set_high(&mut self) -> Result<(), Self::Error>;
    fn set_low(&mut self) -> Result<(), Self::Error>;
    fn toggle(&mut self) -> Result<(), Self::Error>;
    fn read_level(&self) -> Result<GpioLevel, Self::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioLevel {
    Low,
    High,
}
