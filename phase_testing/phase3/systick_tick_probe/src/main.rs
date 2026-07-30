#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};
use entry::entry;
use nucleo_l152re::System;
use specs::common::TaskId;
use specs::kernel::TaskPriority;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 2048] = [0; 2048];

extern "C" fn dummy_task(_arg: *mut ()) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool);

    // single task -> reschedule always reselects it, so PendSV never fires (safe before #40 lands)
    system
        .spawn_task(
            TaskId(1),
            TaskPriority::default(),
            1024,
            dummy_task,
            null_mut(),
        )
        .unwrap();

    let system = unsafe { system.install_as_tick_source() };
    let _ = system;

    loop {
        core::hint::spin_loop();
    }
}
