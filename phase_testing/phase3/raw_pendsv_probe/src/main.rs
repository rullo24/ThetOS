#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};

// local imports
use cortex_m::common::interrupts::enable_interrupts;
use cortex_m::v7m::{set_current_task_tcb, set_next_task_psp};
use cortex_m::V7mContextSwitch;
use entry::entry;
use nucleo_l152re as _;
use specs::arch::ContextSwitch;

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

    // raw, minimal PendSV trigger -> no function calls, no read_volatile/write_volatile
    // wrapper machinery, no ub_checks. just the bare 4 instructions ARM needs.
    unsafe {
        core::arch::asm!(
            "ldr r0, =0xE000ED04",
            "ldr r1, [r0]",
            "orr r1, r1, #0x10000000",
            "str r1, [r0]",
            out("r0") _,
            out("r1") _,
        );
    }

    loop {
        core::hint::spin_loop();
    }
}
