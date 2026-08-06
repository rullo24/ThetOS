/// wait for interrupt -> sleeps the core until the next exception/interrupt wakes it
#[inline]
pub fn wfi() {
    unsafe { core::arch::asm!("wfi") }
}
