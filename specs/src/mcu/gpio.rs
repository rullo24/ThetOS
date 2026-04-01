
use core::result::Result;

pub trait GpioPin {
    type Error: core::fmt::Debug;

    fn set_direction(&mut self, direction: GpioDirection) -> Result<(), Self::Error>;
    fn set_pull(&mut self, pull: GpioPull) -> Result<(), Self::Error>;
    fn set_high(&mut self) -> Result<(), Self::Error>;
    fn set_low(&mut self) -> Result<(), Self::Error>;
    fn toggle(&mut self) -> Result<(), Self::Error>;
    fn read_level(&self) -> Result<GpioLevel, Self::Error>;
}

/// direction of a GPIO pin
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
