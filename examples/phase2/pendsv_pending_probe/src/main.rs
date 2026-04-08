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

static mut STACK_POOL: [u8; 1024] = [0; 1024];

#[entry]
fn app_main() -> ! {
    let mut x: u32 = 0;
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let system = System::new_with_pool(p_stack_pool);
    
    unsafe { core::arch::asm!("cpsid i", options(nomem, nostack)); } // disable interrupts
    system.request_pendsv_pending(); // request PendSV pending (check in GDB)
    unsafe { core::arch::asm!("cpsie i", options(nomem, nostack)); } // enable interrupts

    // to avoid exit
    loop {
        x = x.wrapping_add(5);
    }
}