#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;

// local imports
use entry::entry;
use nucleo_l152re::System;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn app_main() -> ! {
    let mut x: u32 = 0;
    let system = System::new();
    system.request_pendsv_pending(); // request PendSV pending (check in GDB)

    // to avoid exit
    loop {
        x = x.wrapping_add(5);
    }
}