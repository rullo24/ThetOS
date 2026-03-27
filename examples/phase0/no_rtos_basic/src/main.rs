#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;

// local imports
use entry::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn app_main() -> ! {
    let mut x: u32 = 0;
    loop {
        x = x.wrapping_add(5);
    }
}