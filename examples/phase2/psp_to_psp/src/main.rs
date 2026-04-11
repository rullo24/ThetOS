#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};
use core::mem::MaybeUninit;

// local imports
use specs::arch::ContextSwitch;
use entry::entry;
use cortex_m::V7mContextSwitch;
use cortex_m::v7m::{set_current_task_tcb, set_next_task_psp, V7mTaskContext};
use nucleo_l152re as _;
use cortex_m::common::interrupts::enable_interrupts;

#[repr(C, align(8))]
struct TaskStackPool([u8; 2048]);
static mut TASK_STACK_A: TaskStackPool = TaskStackPool([0; 2048]);
static mut TASK_STACK_B: TaskStackPool = TaskStackPool([0; 2048]);
static mut CTX_A: MaybeUninit<V7mTaskContext> = MaybeUninit::uninit();
static mut CTX_B: MaybeUninit<V7mTaskContext> = MaybeUninit::uninit();

static mut HEARTBEAT_A: u32 = 0; // track if state/registers copied correctly
static mut HEARTBEAT_B: u32 = 0; // track if state/registers copied correctly

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// DESCRIPTION
/// task A entry point
#[no_mangle]
extern "C" fn task_a(_arg: *mut ()) -> ! {
    let ctx_switch = V7mContextSwitch;

    loop {
        unsafe {
            HEARTBEAT_A = HEARTBEAT_A.wrapping_add(1); // increment from last
            let p_task_a = addr_of_mut!(CTX_A).cast::<V7mTaskContext>();
            let p_task_b = addr_of_mut!(CTX_B).cast::<V7mTaskContext>();
            set_next_task_psp((*p_task_b).sp);
            set_current_task_tcb(p_task_a);
            ctx_switch.trigger_pendsv_switch();
        }
    }

}

/// DESCRIPTION
/// task B entry point
#[no_mangle]
extern "C" fn task_b(_arg: *mut ()) -> ! {
    let ctx_switch = V7mContextSwitch;

    loop {
        unsafe {
            HEARTBEAT_B = HEARTBEAT_B.wrapping_add(1); // increment from last
            let p_task_a = addr_of_mut!(CTX_A).cast::<V7mTaskContext>();
            let p_task_b = addr_of_mut!(CTX_B).cast::<V7mTaskContext>();
            set_next_task_psp((*p_task_b).sp);
            set_current_task_tcb(p_task_a);
            ctx_switch.trigger_pendsv_switch();
        }
    }
}

#[entry]
fn app_main() -> ! {

    // define stack pools
    let pool_a = unsafe { &mut (*addr_of_mut!(TASK_STACK_A)).0 };
    let pool_b = unsafe { &mut (*addr_of_mut!(TASK_STACK_B)).0 };

    // define stack limits and top pointers
    let limit_a = pool_a.as_mut_ptr();
    let top_a = limit_a.wrapping_add(pool_a.len());
    let limit_b = pool_b.as_mut_ptr();
    let top_b = limit_b.wrapping_add(pool_b.len());
    let ctx_switch = V7mContextSwitch;
    
    // initialise task context A
    let ctx_a = match ctx_switch.initialise_task_context(
        top_a,
        limit_a,
        task_a,
        null_mut(),
    ) {
        Ok(c) => c,
        Err(e) => panic!("failed to init task context A: {:?}", e),
    };

    // initialise task context B
    let ctx_b = match ctx_switch.initialise_task_context(
        top_b,
        limit_b,
        task_b,
        null_mut(),
    ) {
        Ok(c) => c,
        Err(e) => panic!("failed to init task context B: {:?}", e),
    };

    // set task context pointers and PSP/TCB
    unsafe {
        addr_of_mut!(CTX_A).cast::<V7mTaskContext>().write(ctx_a);
        addr_of_mut!(CTX_B).cast::<V7mTaskContext>().write(ctx_b);
        set_next_task_psp(ctx_a.sp);
        set_current_task_tcb(null_mut());
        enable_interrupts();
    }

    // trigger first context switch to task A (MSP -> PSP)
    ctx_switch.trigger_pendsv_switch();

    loop {
        core::hint::spin_loop();
    }

}
