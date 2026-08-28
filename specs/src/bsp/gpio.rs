use core::result::Result;

// used for local typestate
mod sealed {
    pub trait Sealed {}
}

// typestate marke
pub trait PinMode: sealed::Sealed {}

// direction markers
pub struct Uninit; // no direction (pre-config state)
pub struct Input;
pub struct Output;

impl sealed::Sealed for Uninit {}
impl sealed::Sealed for Input {}
impl sealed::Sealed for Output {}
impl PinMode for Uninit {}
impl PinMode for Input {}
impl PinMode for Output {}

// logic level of a GPIO line
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GpioLevel {
    Low,
    High,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PullState {
    HighZ, // floating
    PullUp,
    PullDown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputStyle {
    PushPull,  // active drive HIGH or LOW
    OpenDrain, // active drive LOW (relies on pull-up for HIGH)
}

pub trait UninitPin {
    type Input: InputPin;
    type Output: OutputPin;

    /// DESCRIPTION
    /// Configures the pin in INPUT direction
    fn into_input(self, pull: PullState) -> Self::Input;

    /// DESCRIPTION
    /// Configures the pin in OUTPUT direction
    fn into_output(self, style: OutputStyle) -> Self::Output;
}

pub trait InputPin {
    type Error: core::fmt::Debug;

    /// DESCRIPTION
    /// Reads the current logical level of the pin and returns it
    fn read(&self) -> Result<GpioLevel, Self::Error>;
}

pub trait OutputPin {
    type Error: core::fmt::Debug;

    /// DESCRIPTION
    /// Sets the logical level of the pin to a GpioLevel
    fn set(&mut self, level: GpioLevel) -> Result<(), Self::Error>;
}
