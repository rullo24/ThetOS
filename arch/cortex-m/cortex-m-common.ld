/* simplistic, common linker script -> need to provide MEMORY (FLASH + RAM) in memory.x"
/* startup must define Reset, populate .vector_table, and init .data/.bss before Rust code runs. */

/* MEMORY {} - defined in memory.x elsewhere (mcu/) */

/* defining start of vector table */
_estack = ORIGIN(RAM) + LENGTH(RAM); /* (vector table word 0) */
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

  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
    *(.ARM.extab .ARM.extab.*);
  }
}