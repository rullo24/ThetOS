#![no_std]

// link against MCU startup & vector table
use stm32l152ret6 as _;

pub mod gpio;
pub mod limits;
pub mod system;
pub mod system_timer;

pub use gpio::*; // all GPIO pins + the board-facing GPIO contract
pub use limits::MAX_TASKS;
pub use system::{System, SystemInitError};
pub use system_timer::NucleoSystemTimer;

// kernel/task types a consumer needs, so `specs` stays an internal crate
pub use specs::common::TaskId;
pub use specs::kernel::TaskPriority;
