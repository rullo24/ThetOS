#![no_std]
#![no_main]

use core::panic::PanicInfo;
use entry::entry;
use nucleo_l152re::System;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 4096] = [0; 4096];

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let system = System::new_with_pool(p_stack_pool);

    // OUTPUT direction on LED GPIO
}
