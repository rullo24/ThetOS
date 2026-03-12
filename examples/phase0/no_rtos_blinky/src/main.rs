#![no_main]
#![no_std]

// INTERNAL DEPENDENCIES

// EXTERNAL DEPENDENCIES
use panic_halt as _; // a panic handler must be included (even if unused)
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {} // infinite loop to keep program running
}
