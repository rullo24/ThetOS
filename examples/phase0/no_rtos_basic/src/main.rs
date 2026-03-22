#![no_std]
#![no_main]

use core::panic::PanicInfo;
use entry::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry(bsp = nucleo_l152re)]
fn app_main() -> ! {
    let mut x: u32 = 0;
    loop {
        x = x.wrapping_add(7);
    }
}