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

static mut STACK_POOL: [u8; 2048] = [0; 2048];
static mut COUNTER_A: u32 = 0;
static mut COUNTER_B: u32 = 0;
// counts every delay_ms() that came back NoRunnableTask (would-be simultaneous block) ->
// proves the "everyone wants to sleep" case is actually hit, not just theorised. There is
// no idle task, so block_current_task_until() refuses to block the last runnable task and
// hands the error back instead; the caller here just treats that as "nothing to do yet".
static mut DELAY_REFUSED_COUNT: u32 = 0;

extern "C" fn task_a(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_A = COUNTER_A.wrapping_add(1);
            if system::delay_ms(100).is_err() {
                DELAY_REFUSED_COUNT = DELAY_REFUSED_COUNT.wrapping_add(1);
            }
        }
    }
}

extern "C" fn task_b(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            COUNTER_B = COUNTER_B.wrapping_add(1);
            if system::delay_ms(100).is_err() {
                DELAY_REFUSED_COUNT = DELAY_REFUSED_COUNT.wrapping_add(1);
            }
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    // only 2 tasks, same priority, same delay -> both frequently want to be blocked at once
    system
        .spawn_task(TaskId(1), TaskPriority::default(), 1024, task_a, null_mut())
        .unwrap();
    system
        .spawn_task(TaskId(2), TaskPriority::default(), 1024, task_b, null_mut())
        .unwrap();

    // run RTOS loop -> installs tick source and starts SysTick internally
    system.run();
}
