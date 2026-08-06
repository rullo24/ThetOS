#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut, read_volatile, write_volatile};
use entry::entry;
use nucleo_l152re::System;
use specs::common::TaskId;
use specs::kernel::TaskPriority;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 4096] = [0; 4096];
#[no_mangle]
pub static mut COUNTER_A: u32 = 0;
#[no_mangle]
pub static mut COUNTER_B: u32 = 0;
#[no_mangle]
pub static mut COUNTER_C: u32 = 0;

extern "C" fn task_a(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            write_volatile(addr_of_mut!(COUNTER_A), read_volatile(addr_of_mut!(COUNTER_A)).wrapping_add(1));
        }
    }
}

extern "C" fn task_b(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            write_volatile(addr_of_mut!(COUNTER_B), read_volatile(addr_of_mut!(COUNTER_B)).wrapping_add(1));
        }
    }
}

extern "C" fn task_c(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            write_volatile(addr_of_mut!(COUNTER_C), read_volatile(addr_of_mut!(COUNTER_C)).wrapping_add(1));
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool);

    // spawn 3x tasks for scheduling
    system
        .spawn_task(TaskId(1), TaskPriority::default(), 1024, task_a, null_mut())
        .unwrap();
    system
        .spawn_task(TaskId(2), TaskPriority::default(), 1024, task_b, null_mut())
        .unwrap();
    system
        .spawn_task(TaskId(3), TaskPriority::default(), 1024, task_c, null_mut())
        .unwrap();

    // run RTOS loop -> installs tick source and starts SysTick internally
    system.run();
}
