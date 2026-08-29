#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};
use thetos_entry::entry;
use nucleo_l152re::{System, TaskId, TaskPriority};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 2048] = [0; 2048];
static mut COUNTER: u32 = 0;

extern "C" fn dummy_task(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER = COUNTER.wrapping_add(1);
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

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

    // run RTOS loop -> installs tick source and starts SysTick internally
    system.run();
}
