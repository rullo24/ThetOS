#![no_std]

// link against MCU startup & vector table
use stm32l152ret6 as _;

pub mod gpio;
pub mod limits;
pub mod system;
pub mod system_timer;

pub use gpio::*; // export all GPIO pins
pub use limits::MAX_TASKS;
pub use system::System;
pub use system_timer::NucleoSystemTimer;
