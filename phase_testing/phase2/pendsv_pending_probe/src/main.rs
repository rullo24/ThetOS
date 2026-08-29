#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

// local imports
use entry::entry;
use nucleo_l152re::System;
use cortex_m::common::interrupts::{disable_interrupts, enable_interrupts};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 1024] = [0; 1024];

#[entry]
fn app_main() -> ! {
    let mut x: u32 = 0;
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    disable_interrupts();
    system.request_pendsv_pending().unwrap(); // request PendSV pending (check in GDB)
    enable_interrupts();

    // to avoid exit
    loop {
        x = x.wrapping_add(5);
    }
}