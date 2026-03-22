
use core::result::Result;

pub trait GpioPin {
    type Error: core::fmt::Debug;

    fn setDirection(&mut self, direction: GpioDirection) -> Result<(), Self::Error>;
    fn setHigh(&mut self) -> Result<(), Self::Error>;
    fn setLow(&mut self) -> Result<(), Self::Error>;

    // TODO: add more methods as required by hardware

}

/// direction of a GPIO pin
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioDirection {
    Input,
    Output,
}