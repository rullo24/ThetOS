#![no_std]

// link against MCU startup & vector table
use stm32l152ret6 as _;

pub mod system;
pub use system::System;