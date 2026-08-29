#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};
use entry::entry;
use nucleo_l152re::{system, System};
use specs::common::TaskId;
use specs::kernel::TaskPriority;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 3072] = [0; 3072];
static mut COUNTER_HIGH: u32 = 0;
static mut COUNTER_MID: u32 = 0;
static mut COUNTER_LOW: u32 = 0;

// priority 0 (highest) -> wins every reschedule while ready, but delays after each turn so lower-priority tasks get a real window instead of being starved outright.
extern "C" fn task_high(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_HIGH = COUNTER_HIGH.wrapping_add(1);
        }
        system::delay_ms(300).unwrap();
    }
}

// priority 15 (middle) -> beats task_low whenever both are ready, but also delays, opening a window where task_low (which never delays) finally gets picked.
extern "C" fn task_mid(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_MID = COUNTER_MID.wrapping_add(1);
        }
        system::delay_ms(50).unwrap();
    }
}

// priority 31 (lowest) -> never delays, so it only ever runs in the windows where neither task_high nor task_mid is ready (i.e. both are currently blocked in delay_ms).
extern "C" fn task_low(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_LOW = COUNTER_LOW.wrapping_add(1);
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    system
        .spawn_task(
            TaskId(1),
            TaskPriority::new(0).unwrap(),
            1024,
            task_high,
            null_mut(),
        )
        .unwrap();
    system
        .spawn_task(
            TaskId(2),
            TaskPriority::new(15).unwrap(),
            1024,
            task_mid,
            null_mut(),
        )
        .unwrap();
    system
        .spawn_task(
            TaskId(3),
            TaskPriority::new(31).unwrap(),
            1024,
            task_low,
            null_mut(),
        )
        .unwrap();

    // run RTOS loop -> installs tick source and starts SysTick internally
    system.run();
}
