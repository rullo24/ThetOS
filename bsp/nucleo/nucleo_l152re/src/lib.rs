#![no_std]

// link against MCU startup & vector table
use stm32l152ret6 as _;

pub mod system;
pub mod system_timer;
pub mod limits;

pub use system::System;
pub use system_timer::NucleoSystemTimer;
pub use limits::MAX_TASKS;