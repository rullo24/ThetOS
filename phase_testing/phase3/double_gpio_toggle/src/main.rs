#![no_std]
#![no_main]

// STD INCLUDES
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};

// USER INCLUDES
use thetos_entry::entry;
use nucleo_l152re::{system, System, PA6, PA7};
use specs::bsp::{GpioLevel, OutputPin, OutputStyle, UninitPin};
use specs::common::TaskId;
use specs::kernel::TaskPriority;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 4096] = [0; 4096];

extern "C" fn toggle_task_a(_arg: *mut ()) -> ! {
    let mut pa6 = PA6.into_output(OutputStyle::PushPull);

    loop {
        pa6.set(GpioLevel::High);
        system::delay_ms(1000).unwrap();
        pa6.set(GpioLevel::Low);
        system::delay_ms(1000).unwrap();
    }
}

extern "C" fn toggle_task_b(_arg: *mut ()) -> ! {
    let mut pa7 = PA7.into_output(OutputStyle::PushPull);

    loop {
        pa7.set(GpioLevel::High);
        system::delay_ms(1000).unwrap();
        pa7.set(GpioLevel::Low);
        system::delay_ms(1000).unwrap();
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    // define tasks
    system
        .spawn_task(
            TaskId(1),
            TaskPriority::default(),
            2048,
            toggle_task_a,
            null_mut(),
        )
        .unwrap();
    system
        .spawn_task(
            TaskId(2),
            TaskPriority::default(),
            2048,
            toggle_task_b,
            null_mut(),
        )
        .unwrap();

    // start scheduler
    system.run();
}
