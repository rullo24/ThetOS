use core::result::Result;

/// low-level MCU GPIO capability contract.
pub trait GpioPortPin {
    type Error: core::fmt::Debug;

    fn configure_direction(&mut self, direction: GpioDirection) -> Result<(), Self::Error>;
    fn configure_pull(&mut self, pull: GpioPull) -> Result<(), Self::Error>;
    fn write_high(&mut self) -> Result<(), Self::Error>;
    fn write_low(&mut self) -> Result<(), Self::Error>;
    fn read_level(&self) -> Result<GpioLevel, Self::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioDirection {
    Input,
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioPull {
    None,
    Up,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioLevel {
    Low,
    High,
}
