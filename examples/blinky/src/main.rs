// (examples/blinky/src/main.rs) will look exactly like a 
// real-world application. It doesn't "see" the arch or 
// "specs" folders directly, it sees them through the kernel.

#![no_std]
#![no_main]

// kernel re-exports the architecture and traits for the user
use kernel::prelude::*; 

#[no_main]
fn main() -> ! {
    // hardware is auto-detected by the kernel based on build target
    let mut cpu = arch::init(); 

    // user only cares about the RTOS API (not the hardware)
    let mut sched = Scheduler::new();
    
    sched.add_task(my_logic);
    sched.start();
}