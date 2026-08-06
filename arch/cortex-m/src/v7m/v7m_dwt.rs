/// DWT cycle counter helpers -> lets callers time real hardware execution (e.g. checking a tick handler against its SysTick budget).
use core::ptr::{read_volatile, write_volatile};

const DEMCR: *mut u32 = 0xE000_EDFC as *mut u32; // Debug Exception and Monitor Control Register
const DEMCR_TRCENA: u32 = 1 << 24; // enables the DWT unit

const DWT_CTRL: *mut u32 = 0xE000_1000 as *mut u32;
const DWT_CTRL_CYCCNTENA: u32 = 1 << 0; // enables the free-running cycle counter

const DWT_CYCCNT: *mut u32 = 0xE000_1004 as *mut u32; // free-running cycle counter, wraps at u32::MAX

/// enable the DWT cycle counter -> call once, early at boot
pub unsafe fn init_cycle_counter() {
    write_volatile(DEMCR, read_volatile(DEMCR) | DEMCR_TRCENA);
    write_volatile(DWT_CYCCNT, 0);
    write_volatile(DWT_CTRL, read_volatile(DWT_CTRL) | DWT_CTRL_CYCCNTENA);
}

/// read the current free-running cycle count
pub unsafe fn read_cycle_counter() -> u32 {
    read_volatile(DWT_CYCCNT)
}
