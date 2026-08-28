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

static mut STACK_POOL: [u8; 4096] = [0; 4096];

extern "C" fn blink_onboard(_arg: *mut ()) -> ! {
    loop {} // TODO: replace with blink logic
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let system = System::new_with_pool(p_stack_pool);

    // OUTPUT direction on LED GPIO

    // define tasks
    system
        .spawn_task(
            TaskId(1),
            TaskPriority::default(),
            4096,
            blink_onboard,
            null_mut(),
        )
        .unwrap();

    // start scheduler
    system.run();
}
