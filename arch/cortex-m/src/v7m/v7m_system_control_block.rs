// REFERENCES:
// https://developer.arm.com/documentation/dui0552/a/cortex-m3-peripherals/system-control-block?lang=en

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{compiler_fence, Ordering};

// https://developer.arm.com/documentation/dui0552/a/cortex-m3-peripherals/system-control-block/interrupt-control-and-state-register
pub const SCB_ICSR: *mut u32 = 0xE000ED04 as *mut u32; // ICSR (Interrupt Control and State Register) -> SCB base 0xE000_ED00 + 0x04
pub const ICSR_PENDSV_SET: u32 = 1 << 28; // bit 28 is set to 1 -> request PendSV interrupt

// https://developer.arm.com/documentation/dui0552/latest/cortex-m3-peripherals/system-control-block/system-handler-priority-registers
pub const SCB_SHPR3: *mut u32 = 0xE000ED20 as *mut u32; // SHPR3 (System Handler Priority Register 3) -> SCB base 0xE000_ED00 + 0x20
pub const NVIC_PRIO_BITS: u8 = 4; // captured from ST RM0038 Reference Manual (pg 230/904)
pub const LOWEST_PRIORITY: u8 = (((1u16 << NVIC_PRIO_BITS) - 1) as u8) << (8 - NVIC_PRIO_BITS);
const SHPR3_PRI14_PENDSV_SHIFT: u32 = 16; // priority of system handler 14, PendSV
const SHPR3_PRI15_SYSTICK_SHIFT: u32 = 24; // priority of system handler 15, SysTick exception

/// DESCRIPTION
/// request PendSV pending by writing HIGH to ICSR register
pub unsafe fn request_pendsv_pending() {
    compiler_fence(Ordering::SeqCst); // ensure prior MMIO stores complete before later instructions run
    let p_icsr: u32 = read_volatile(SCB_ICSR);
    write_volatile(SCB_ICSR, p_icsr | ICSR_PENDSV_SET); // set PendSV bit
    core::arch::asm!("dsb"); // ensure prior MMIO stores complete before later instructions run
    core::arch::asm!("isb"); // flush the CPU pipeline so the next instruction is fetched fresh after touching system registers
}

/// DESCRIPTION
/// set PendSV and SysTick to the same lowest priority (standard Cortex-M RTOS convention)
pub unsafe fn configure_kernel_interrupt_priorities() {
    let priority = LOWEST_PRIORITY as u32;
    let value = (priority << SHPR3_PRI14_PENDSV_SHIFT) | (priority << SHPR3_PRI15_SYSTICK_SHIFT);
    let mask = (0xFFu32 << SHPR3_PRI14_PENDSV_SHIFT) | (0xFFu32 << SHPR3_PRI15_SYSTICK_SHIFT);
    let existing = read_volatile(SCB_SHPR3);
    write_volatile(SCB_SHPR3, (existing & !mask) | value);
}
