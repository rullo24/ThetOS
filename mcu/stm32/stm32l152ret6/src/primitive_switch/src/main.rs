#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};

// local imports
use specs::arch::ContextSwitch;
use thetos_entry::entry;
use cortex_m::V7mContextSwitch;
use cortex_m::v7m::{set_current_task_tcb, set_next_task_psp};
use nucleo_l152re as _;
use cortex_m::common::interrupts::enable_interrupts;

#[repr(C, align(8))]
struct TaskStackPool([u8; 2048]);
static mut TASK_STACK_POOL: TaskStackPool = TaskStackPool([0; 2048]);
static mut TASK_HEARTBEAT: u32 = 0;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// DESCRIPTION
/// task entry point -> increment heartbeat and spin loop
#[no_mangle]
extern "C" fn task_entry(_arg: *mut ()) -> ! {
    unsafe {
        TASK_HEARTBEAT = TASK_HEARTBEAT.wrapping_add(1);
    }

    loop {
        core::hint::spin_loop();
    }
}

#[entry]
fn app_main() -> ! {
    
    // defining stack resources
    let pool = unsafe { &mut (*addr_of_mut!(TASK_STACK_POOL)).0 };
    let stack_limit = pool.as_mut_ptr();
    let stack_top = stack_limit.wrapping_add(pool.len());
    
    let ctx_switch = V7mContextSwitch;
    let ctx = match ctx_switch.initialise_task_context(
        stack_top,
        stack_limit,
        task_entry, // entry point (func ptr)
        null_mut(), // no args
    ) {
        Ok(c) => c,
        Err(e) => panic!("failed to init task context: {:?}", e),
    };
    
    unsafe {
        set_next_task_psp(ctx.sp); // set next task PSP -> must match `V7mTaskContext.sp`
        set_current_task_tcb(null_mut()); // no current task -> NULL
        enable_interrupts(); // ensure interrupts are enabled before context switch (should already be enabled)
    }

    // trigger context switch to change task
    ctx_switch.trigger_pendsv_switch(); 

    loop {
        core::hint::spin_loop();
    }
}