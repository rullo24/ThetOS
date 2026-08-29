pub mod pin;
mod pins;
pub mod port;

pub use pin::Pin;
pub use pins::*;
pub use port::{GpioPort, PortA, PortB, PortC, PortD, PortE, PortH};

// re-export the board-facing GPIO contract so consumers never name `specs` directly
pub use specs::bsp::gpio::{
    GpioLevel, Input, InputPin, Output, OutputPin, OutputStyle, PinMode, PullState, Uninit, UninitPin,
};
