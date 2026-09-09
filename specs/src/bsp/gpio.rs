// used for local typestate
mod sealed {
    pub trait Sealed {}
}

// typestate marker
pub trait PinMode: sealed::Sealed {}

// pin mode markers
pub struct Uninit; // no mode chosen (pre-config state)
pub struct Input;
pub struct Output;
pub struct Alternate; // driven by an on-chip peripheral, not application code

impl sealed::Sealed for Uninit {}
impl sealed::Sealed for Input {}
impl sealed::Sealed for Output {}
impl sealed::Sealed for Alternate {}
impl PinMode for Uninit {}
impl PinMode for Input {}
impl PinMode for Output {}
impl PinMode for Alternate {}

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
    /// DESCRIPTION
    /// Reads the current logical level of the pin and returns it
    fn read(&self) -> GpioLevel;
}

pub trait OutputPin {
    /// DESCRIPTION
    /// Sets the logical level of the pin to a GpioLevel
    fn set(&mut self, level: GpioLevel);
}
