/*
  * Common Cortex-M layout (M0/M0+, M3, M4, M4F): link memory.x first, then this file.
  * Startup must define Reset, emit .vector_table, and copy .data / zero .bss before Rust code runs.
  * MEMORY {} - defined in memory.x (mcu/)
  * Not here: vector table/VTOR/boot offset, FPU/ABI (target + startup), MPU/TrustZone, NVIC/SysTick,
  * extra RAM regions or XIP (use memory.x + extra SECTIONS), unwinding (we discard exidx/extab).
*/

/* defining start of vector table */
_estack = ORIGIN(RAM) + LENGTH(RAM); /* (vector table word 0) */
ASSERT((_estack % 8) == 0, "initial MSP must be 8-byte aligned (AAPCS)");
ENTRY(Reset); /* vector table word 1 */

SECTIONS
{
  /* First sector: startup should emit .vector_table (typically from asm). */
  .vector_table :
  {
    KEEP(*(.vector_table))
  } > FLASH

  .text :
  {
    *(.text .text.*)
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*)
  } > FLASH

  /* Initialised data: VMA in RAM, LMA in FLASH */
  .data : ALIGN(4)
  {
    __sdata = .;
    *(.data .data.*);
    . = ALIGN(4);
    __edata = .;
  } > RAM AT> FLASH

  __sidata = LOADADDR(.data);

  .bss (NOLOAD) : ALIGN(4)
  {
    __sbss = .;
    *(.bss .bss.*);
    *(COMMON);
    . = ALIGN(4);
    __ebss = .;
  } > RAM

  /* discard unwinding tables */
  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
    *(.ARM.extab .ARM.extab.*);
  }
}