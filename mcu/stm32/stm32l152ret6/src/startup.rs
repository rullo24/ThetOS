//! Reset path, vector table, and RAM init for STM32L152RE (Cortex-M3).

use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

// Must match `MEMORY { RAM }` end in `linker.ld` (ORIGIN + LENGTH).
const RAM_ORIGIN: u32 = 0x2000_0000;
const RAM_LENGTH_BYTES: u32 = 80 * 1024;
const ESTACK: u32 = RAM_ORIGIN + RAM_LENGTH_BYTES;

unsafe extern "C" {
    static mut __sdata: u8;
    static mut __edata: u8;
    static mut __sbss: u8;
    static mut __ebss: u8;
    static __sidata: u8;
}

unsafe extern "C" {
    fn main() -> !;
}

#[no_mangle]
pub extern "C" fn Default_Handler() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn Reset() -> ! {
    unsafe {
        copy_data();
        zero_bss();
        main();
    }
}

// copy the data from flash to RAM
unsafe fn copy_data() {
    let mut dst = addr_of_mut!(__sdata);
    let dst_end = addr_of_mut!(__edata);
    let mut src = addr_of!(__sidata);
    while (dst as usize) < (dst_end as usize) {
        write_volatile(dst, read_volatile(src));
        dst = dst.wrapping_add(1);
        src = src.wrapping_add(1);
    }
}

// zero the bss sections
unsafe fn zero_bss() {
    let mut p = addr_of_mut!(__sbss);
    let end = addr_of_mut!(__ebss);
    while (p as usize) < (end as usize) {
        write_volatile(p, 0);
        p = p.wrapping_add(1);
    }
}

core::arch::global_asm!(
    r#"
    .section .vector_table,"a",%progbits   // Put following bytes in section `.vector_table`, allocatable, program bits (normal flash data).
    .align 2                                // Align location counter to 4 bytes (2^2); vectors must be word-aligned.
    .word {estack}                          // Word 0: initial MSP value (top of stack), not a branch target.
    .word Reset                             // Word 1: reset vector — address of `Reset` (Thumb, LSB set by linker).
    .rept 46                                // Emit the next directive 46 times (remaining core + NVIC slots for this chip).
    .word Default_Handler                  // Each slot: handler address; shared default until you override per IRQ.
    .endr                                   // End of `.rept` block.
    "#,
    estack = const ESTACK,                  // Rust `const` folded into the first `.word` (must match `_estack` / RAM end).
);