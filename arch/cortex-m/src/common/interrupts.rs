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
            "mrs {p}, primask", // load PRIMASK reg into p var
            p = out(reg) primask_val, // output -> general purpose reg
            options(nomem, nostack, preserves_flags) // constrain side effects of LLVM
        );
        primask_val == 0 // if PRIMASK is 0, interrupts are available
    }
}

/// DESCRIPTION
/// snapshot of PRIMASK IRQ-unmask state stored and interrupts disabled
pub fn irsq_available_snapshot_and_disable() -> bool {
    let saved_primask: u32;
    unsafe {
        core::arch::asm!(
            "mrs {saved}, PRIMASK",
            "cpsid i",
            saved = out(reg) saved_primask,
            options(nomem, nostack, preserves_flags),
        );
    }
    saved_primask == 0 // return true if interrupts were available
}

/// DESCRIPTION
/// set the PRIMASK IRQ-unmask state
pub fn set_irqs_primask(state: bool) {
    unsafe {
        core::arch::asm!(
            "msr primask, {state}",
            state = in(reg) (!state) as u32, // PRIMASK 0=unmasked -> unmask(true) must write 0, not 1
            options(nomem, nostack, preserves_flags),
        );
    }
}