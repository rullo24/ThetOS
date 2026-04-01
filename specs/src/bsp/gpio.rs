use core::result::Result;

/// board-facing GPIO resource contract exposed to application code.
pub trait BoardGpioPin {
    type Error: core::fmt::Debug;

    fn set_level(&mut self, level: GpioLevel) -> Result<(), Self::Error>;
    fn read_level(&self) -> Result<GpioLevel, Self::Error>;
    fn toggle(&mut self) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioLevel {
    Low,
    High,
}
