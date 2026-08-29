#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};
use thetos_entry::entry;
use nucleo_l152re::{system, System, TaskId, TaskPriority};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 5120] = [0; 5120];
static mut COUNTER_P0: u32 = 0;
static mut COUNTER_P7: u32 = 0;
static mut COUNTER_P15: u32 = 0;
static mut COUNTER_P23: u32 = 0;
static mut COUNTER_P31: u32 = 0;

// priority 0 (highest) -> should dominate every reschedule while ready; delays each cycle so lower tiers get a real window.
extern "C" fn task_p0(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_P0 = COUNTER_P0.wrapping_add(1);
        }
        system::delay_ms(400).unwrap();
    }
}

// priority 7 -> beats p15/p23/p31 whenever ready, loses to p0; delays each cycle too.
extern "C" fn task_p7(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_P7 = COUNTER_P7.wrapping_add(1);
        }
        system::delay_ms(200).unwrap();
    }
}

// priority 15 -> beats p23/p31 whenever ready, loses to p0/p7; delays each cycle too.
extern "C" fn task_p15(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_P15 = COUNTER_P15.wrapping_add(1);
        }
        system::delay_ms(100).unwrap();
    }
}

// priority 23 -> beats p31 whenever ready, loses to everything above it; shortest delay of the four that block.
extern "C" fn task_p23(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_P23 = COUNTER_P23.wrapping_add(1);
        }
        system::delay_ms(50).unwrap();
    }
}

// priority 31 (lowest) -> never delays, so it only runs when every higher tier above it is simultaneously blocked.
extern "C" fn task_p31(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_P31 = COUNTER_P31.wrapping_add(1);
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    system.spawn_task(TaskId(1), TaskPriority::new(0).unwrap(), 1024, task_p0, null_mut()).unwrap();
    system.spawn_task(TaskId(2), TaskPriority::new(7).unwrap(), 1024, task_p7, null_mut()).unwrap();
    system.spawn_task(TaskId(3), TaskPriority::new(15).unwrap(), 1024, task_p15, null_mut()).unwrap();
    system.spawn_task(TaskId(4), TaskPriority::new(23).unwrap(), 1024, task_p23, null_mut()).unwrap();
    system.spawn_task(TaskId(5), TaskPriority::new(31).unwrap(), 1024, task_p31, null_mut()).unwrap();

    // run RTOS loop -> installs tick source and starts SysTick internally
    system.run();
}
