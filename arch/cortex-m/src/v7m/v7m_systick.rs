// REFERENCE: arm developer -> Cortex-M3 Devices Generic User Guide / System Timer, SysTick
// https://developer.arm.com/documentation/dui0552/a/cortex-m3-peripherals/system-timer--systick

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{compiler_fence, Ordering};
use specs::arch::SystemTicker;

// SysTick registers
const SYST_CSR: *mut u32 = 0xE000_E010 as *mut u32;
const SYST_RVR: *mut u32 = 0xE000_E014 as *mut u32;
const SYST_CVR: *mut u32 = 0xE000_E018 as *mut u32;

const SYST_CSR_ENABLE: u32 = 1 << 0; // 0 = counter disabled | 1 = counter enabled
const SYST_CSR_TICKINT: u32 = 1 << 1; // 0 = count to zero DOES NOT assert SysTick exception | 1 = DOES asset SysTick exception
const SYST_CSR_CLKSOURCE: u32 = 1 << 2; // 0 = ext. CLK | 1 = processor CLK

// reuse SCB_ICSR's address -> see v7m_system_control_block.rs.
use super::v7m_system_control_block::SCB_ICSR;
const ICSR_PENDSTCLR: u32 = 1 << 25; // bit 25 is set to 1 -> clear SysTick pending
const SYST_RVR_MASK: u32 = 0x00FF_FFFF; // 24-bit reload value (2^23=8,388,608) after SysTick expire

// software-extended tick counter -> only the ISR touches this (no scheduler logic)
static mut SYSTICK_TICK_COUNT: u64 = 0;

// callback when SysTick occurs -> registed once by BSP on init
static mut SYSTICK_CALLBACK: Option<fn()> = None;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysTickError {
    ReloadOutOfRange,
}

// zero-sized handle onto the SysTick peripheral
#[derive(Clone, Copy)]
pub struct V7mSysTick;

impl SystemTicker for V7mSysTick {
    type Error = SysTickError;

    /// DESCRIPTION
    /// enable the counter with the processor clock as its source
    fn start(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let csr = read_volatile(SYST_CSR);
            write_volatile(SYST_CSR, csr | SYST_CSR_ENABLE | SYST_CSR_CLKSOURCE);
            // counter enabled + processor CLK used
        }
        Ok(())
    }

    /// DESCRIPTION
    /// disable the counter
    fn stop(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let csr = read_volatile(SYST_CSR);
            write_volatile(SYST_CSR, csr & !SYST_CSR_ENABLE); // disable SysTick counter
        }
        Ok(())
    }

    /// DESCRIPTION
    /// set the 24-bit reload value and reset the counter
    fn set_reload(&mut self, ticks: u32) -> Result<(), Self::Error> {
        if ticks & !SYST_RVR_MASK != 0 {
            return Err(SysTickError::ReloadOutOfRange);
        }
        unsafe {
            write_volatile(SYST_RVR, ticks & SYST_RVR_MASK);
            write_volatile(SYST_CVR, 0); // any write clears CVR + COUNTFLAG
        }
        Ok(())
    }

    /// DESCRIPTION
    /// restart the countdown from the top -> reload value is untouched, only CVR is cleared
    fn reset_counter(&mut self) -> Result<(), Self::Error> {
        unsafe {
            write_volatile(SYST_CVR, 0); // any write clears CVR + COUNTFLAG -> next tick reloads from SYST_RVR
        }
        Ok(())
    }

    /// DESCRIPTION
    /// return the software-extended tick count maintained by the ISR
    fn current_tick(&self) -> Result<u64, Self::Error> {
        Ok(unsafe { read_volatile(core::ptr::addr_of!(SYSTICK_TICK_COUNT)) })
    }

    /// DESCRIPTION
    /// enable the SysTick interrupt (TICKINT)
    fn enable_interrupt(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let csr = read_volatile(SYST_CSR);
            write_volatile(SYST_CSR, csr | SYST_CSR_TICKINT);
        }
        Ok(())
    }

    /// DESCRIPTION
    /// disable the SysTick interrupt (TICKINT)
    fn disable_interrupt(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let csr = read_volatile(SYST_CSR);
            write_volatile(SYST_CSR, csr & !SYST_CSR_TICKINT);
        }
        Ok(())
    }

    /// DESCRIPTION
    /// clear a pending SysTick exception request via ICSR PENDSTCLR
    fn clear_pending(&mut self) -> Result<(), Self::Error> {
        unsafe {
            compiler_fence(Ordering::SeqCst);
            write_volatile(SCB_ICSR, ICSR_PENDSTCLR);
            core::arch::asm!("dsb");
            core::arch::asm!("isb");
        }
        Ok(())
    }
}

/// DESCRIPTION
/// register the function to call on every SysTick interrupt -> must be set before enable_interrupt()
pub unsafe fn set_tick_callback(callback: fn()) {
    write_volatile(core::ptr::addr_of_mut!(SYSTICK_CALLBACK), Some(callback));
}

/// DESCRIPTION
/// minimal SysTick ISR -> increments the software tick count only, then invokes BSP-registered callback
#[no_mangle]
pub extern "C" fn SysTick_Handler() {
    unsafe {
        let count = read_volatile(core::ptr::addr_of!(SYSTICK_TICK_COUNT));
        write_volatile(
            core::ptr::addr_of_mut!(SYSTICK_TICK_COUNT),
            count.wrapping_add(1),
        );

        // invokes SysTick callback (if avail)
        if let Some(callback) = read_volatile(core::ptr::addr_of!(SYSTICK_CALLBACK)) {
            callback();
        }
    }
}
