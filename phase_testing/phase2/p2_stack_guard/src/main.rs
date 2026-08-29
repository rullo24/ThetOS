#![no_std]
#![no_main]

// core lib imports
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use core::ptr::write_volatile;

// local imports
use cortex_m::CortexMStackGuard;
use cortex_m::common::interrupts::enable_interrupts;
use thetos_entry::entry;
use nucleo_l152re as _;
use specs::arch::{
    StackGuard,
    StackGuardConfig,
    StackGuardContext,
    StackGuardError,
    StackGuardMode,
    StackGuardState,
};

// must match the canary used in `StackGuardConfig` (GDB can watch this address).
const CANARY_WORD: u32 = 0xDEADBEEF;

#[repr(C, align(8))]
struct StackPool([u8; 2048]);
static mut STACK: StackPool = StackPool([0; 2048]);

/// 0 = not run, 1 = checks passed before corruption, 2 = corruption detected (expected)
#[no_mangle]
static mut GUARD_DEMO_PHASE: u32 = 0;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn app_main() -> ! {
    
    // define stack pool and limits
    let pool = unsafe { &mut (*addr_of_mut!(STACK)).0 };
    let stack_limit = pool.as_mut_ptr();
    let stack_top = stack_limit.wrapping_add(pool.len());

    // creating stack guard context
    let guard = CortexMStackGuard;
    let mut sg_ctx = StackGuardContext {
        stack_top,
        stack_limit,
        state: StackGuardState {
            low_mark: stack_limit,
        },
        config: StackGuardConfig {
            mode: StackGuardMode::Canary,
            canary_word: CANARY_WORD,
        },
    };  

    // initialise stack guard
    if guard.initialise(&mut sg_ctx).is_err() {
        unsafe { GUARD_DEMO_PHASE = 0xff };
        loop {
            core::hint::spin_loop();
        }
    }

    // check for initial state
    if guard.check(&mut sg_ctx).is_ok() {
        unsafe { GUARD_DEMO_PHASE = 1 };
    }   

    // deliberate stack corruption
    unsafe {
        write_volatile(stack_limit.cast::<u32>(), 0);
    }

    // check for corruption
    match guard.check(&mut sg_ctx) {
        Err(StackGuardError::GuardCorrupted) => {
            unsafe { GUARD_DEMO_PHASE = 2 };
        }
        _ => {
            unsafe { GUARD_DEMO_PHASE = 0xfe };
        }
    }

    // enable interrupts and wait for corruption
    enable_interrupts();

    loop {
        core::hint::spin_loop();
    }

}