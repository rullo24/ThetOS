#![no_main]
#![no_std]

// INTERNAL DEPENDENCIES

// EXTERNAL DEPENDENCIES
use core::panic::PanicInfo;
use cortex_m_rt::entry;

#[panic_handler]
#[inline(never)]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    loop {} 
}