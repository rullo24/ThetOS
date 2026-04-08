/// DESCRIPTION
/// disable interrupts
#[inline]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cpsid i", options(nomem, nostack, preserves_flags));
    }
}

/// DESCRIPTION
/// enable interrupts
#[inline]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
    }
}

/// DESCRIPTION
/// snapshot of PRIMASK IRQ-unmask state (may change immediately after return)
pub fn irqs_available_snapshot() -> bool {
    unsafe {
        let primask_val: u32;
        core::arch::asm!(
            "mrs {primask_val}, primask", // load PRIMASK reg into p var
            out(reg) primask_val, // output -> general purpose reg
            options(nomem, nostack, preserves_flags) // constrain side effects of LLVM
        );
        p == 0 // if PRIMASK is 0, interrupts are available
    }
}