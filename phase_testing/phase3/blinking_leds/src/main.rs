#![no_std]
#![no_main]

// STD INCLUDES
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};

// USER INCLUDES
use entry::entry;
use nucleo_l152re::{system, System, PA5};
use specs::bsp::{GpioLevel, OutputPin, OutputStyle, UninitPin};
use specs::common::TaskId;
use specs::kernel::TaskPriority;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 4096] = [0; 4096];

extern "C" fn blink_task(_arg: *mut ()) -> ! {
    // capture LD2 user LED
    let mut ld2 = PA5.into_output(OutputStyle::PushPull);

    loop {
        ld2.set(GpioLevel::High);
        system::delay_ms(1000).unwrap();
        ld2.set(GpioLevel::Low);
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
            4096,
            blink_task,
            null_mut(),
        )
        .unwrap();

    // start scheduler
    system.run();
}
