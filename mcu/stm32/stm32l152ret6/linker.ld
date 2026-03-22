/* Minimal STM32L152RE (512K flash, 80K RAM) -> single file */
/* Startup must define Reset, populate .vector_table, and init .data/.bss before Rust code runs. */

MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM   : ORIGIN = 0x20000000, LENGTH = 80K
}

/* Initial stack (vector word 0) */
_estack = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(Reset);

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

  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
    *(.ARM.extab .ARM.extab.*);
  }
}