#![no_std]
#![no_main]

use core::panic::PanicInfo;

use stm32l152ret6 as _;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut x: u32 = 0;
    loop {
        x = x.wrapping_add(7);
        // Optional: stop GDB here every iteration — set `break main` on the next line after add, or use a separate `#[inline(never)] fn step()` and break there.
    }
}