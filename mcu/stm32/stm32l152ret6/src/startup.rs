/// Reset path, vector table, and RAM init for STM32L152RE (Cortex-M3).

use core::ptr::{
    addr_of, 
    addr_of_mut, 
    read_volatile, 
    write_volatile,
};

// pulled from linker.ld
const RAM_ORIGIN: u32 = 0x2000_0000;
const RAM_LENGTH_BYTES: u32 = 80 * 1024;
const ESTACK: u32 = RAM_ORIGIN + RAM_LENGTH_BYTES;

// defining linker symbols (match linker.ld)
unsafe extern "C" { // importing symbols like this is not memory-layout specific (this is not a struct)
    static mut __sdata: u8;
    static mut __edata: u8;
    static __sidata: u8;
    static mut __sbss: u8;
    static mut __ebss: u8;
}

// defining main function lives elsewhere (we will hand over to this)
unsafe extern "C" {
    fn main() -> !;
}

// default handler for unhandled interrupts
#[no_mangle] // prevent compiler from renaming func
pub extern "C" fn Default_Handler() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// the function that is called on startup/reset of the mcu
#[no_mangle] // prevent compiler from renaming func
pub extern "C" fn Reset() -> ! {
    unsafe {
        copy_data(); // copy data from FLASH to RAM
        zero_bss(); // zero bss section in RAM
        main(); // hand over to user execution code
    }
}

// copy the data from flash to RAM
unsafe fn copy_data() {
    let mut dst = addr_of_mut!(__sdata); // start of data section in RAM
    let dst_end = addr_of_mut!(__edata); // end of data section in RAM
    let mut src = addr_of!(__sidata); // start of data section in FLASH
    while (dst as usize) < (dst_end as usize) { // iterate over each RAM byte
        write_volatile(dst, read_volatile(src)); // write the byte from FLASH to RAM
        dst = dst.wrapping_add(1); // increment the RAM ptr
        src = src.wrapping_add(1); // increment the RAM ptr
    }
}

// zero the bss sections
unsafe fn zero_bss() {
    let mut p_bss_curr = addr_of_mut!(__sbss); // start of bss section in RAM
    let end_bss = addr_of_mut!(__ebss); // end of bss section in RAM
    while (p_bss_curr as usize) < (end_bss as usize) { // iterate over each bss byte
        write_volatile(p_bss_curr, 0); // write 0x0 to current byte
        p_bss_curr = p_bss_curr.wrapping_add(1); // increment ptr
    }
}

// embeds asm for the vector table in flash
core::arch::global_asm!(
    r#"
    .section .vector_table,"a",%progbits    // Put following bytes in section `.vector_table`, allocatable, program bits (normal flash data).
    .align 2                                // Align location counter to 4 bytes (2^2); vectors must be word-aligned.
    .word {estack}                          // Word 0: initial MSP value (top of stack), not a branch target.
    .word Reset                             // Word 1: reset vector — address of `Reset` (Thumb, LSB set by linker). Matches Reset func in asm above.
    .rept 46                                // Emit the next directive 46 times (remaining core + NVIC slots for this chip).
    .word Default_Handler                   // Each slot: handler address; shared default until you override per IRQ.
    .endr                                   // End of `.rept` block.
    "#,
    estack = const ESTACK,                  // matched to {estack} in asm above
);