// REFERENCE: arm developer -> Home / Documentation / IP Products / Processors / Cortex-M / Cortex-M3 / Cortex-M3 Devices Generic User Guide / System Control Block
// https://developer.arm.com/documentation/dui0552/a/cortex-m3-peripherals/system-control-block?lang=en

use core::ptr::{write_volatile, read_volatile};
use core::sync::atomic::{compiler_fence, Ordering};

/// ICSR (Interrupt Control and State Register) -> SCB base 0xE000_ED00 + 0x04.
pub const SCB_ICSR: *mut u32 = 0xE000ED04 as *mut u32;

/// Write 1 to set PendSV pending -> do not set PENDSVCLR in the same write.
pub const ICSR_PENDSV_SET: u32 = 1 << 28; // bit 28 is set to 1 -> request PendSV interrupt

/// DESCRIPTION
/// request PendSV pending by writing HIGH to ICSR register
pub unsafe fn request_pendsv_pending() {
    compiler_fence(Ordering::SeqCst); // ensure prior MMIO stores complete before later instructions run
    let p_icsr: u32 = read_volatile(SCB_ICSR);
    write_volatile(SCB_ICSR, p_icsr | ICSR_PENDSV_SET); // set PendSV bit
    core::arch::asm!("dsb"); // ensure prior MMIO stores complete before later instructions run
    core::arch::asm!("isb"); // flush the CPU pipeline so the next instruction is fetched fresh after touching system registers
}