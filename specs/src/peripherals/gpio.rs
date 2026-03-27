
use core::result::Result;

pub trait GpioPin {
    type Error: core::fmt::Debug;

    fn setDirection(&mut self, direction: GpioDirection) -> Result<(), Self::Error>;
    fn setPull(&mut self, pull: GpioPull) -> Result<(), Self::Error>;
    fn setHigh(&mut self) -> Result<(), Self::Error>;
    fn setLow(&mut self) -> Result<(), Self::Error>;
    fn toggle(&mut self) -> Result<(), Self::Error>;
    fn readLevel(&self) -> Result<GpioLevel, Self::Error>;
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
