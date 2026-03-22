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
    let mut dst: *mut u8 = addr_of_mut!(__sdata); // start of data section in RAM
    let dst_end: *mut u8 = addr_of_mut!(__edata); // end of data section in RAM
    let mut src: *const u8 = addr_of!(__sidata); // start of data section in FLASH
    while (dst as usize) < (dst_end as usize) { // iterate over each RAM byte
        write_volatile(dst, read_volatile(src)); // write the byte from FLASH to RAM
        dst = dst.wrapping_add(1); // increment the RAM ptr
        src = src.wrapping_add(1); // increment the RAM ptr
    }
}

// zero the bss sections
unsafe fn zero_bss() {
    let mut p_bss_curr: *mut u8 = addr_of_mut!(__sbss); // start of bss section in RAM
    let end_bss: *mut u8 = addr_of_mut!(__ebss); // end of bss section in RAM
    while (p_bss_curr as usize) < (end_bss as usize) { // iterate over each bss byte
        write_volatile(p_bss_curr, 0); // write 0x0 to current byte
        p_bss_curr = p_bss_curr.wrapping_add(1); // increment ptr
    }
}

// embeds asm for the vector table in flash
// NOTE: mcu reference manual is Cat.5 (found in table 3 - pg 41/904)
// NOTE: vector table defined in STM32L152xx reference manual, table 51 (pg 235/904)
core::arch::global_asm!(
    r#"
    .section .vector_table,"a",%progbits // put following bytes in section `.vector_table`, allocatable, program bits (normal flash data)
    .align 2                    // align to 4 bytes (thumb code is 2-byte -> align 2*2 = 4 bytes (32-bit words))
    .word {estack}              // 0x000: initial MSP.
    .word Reset                 // 0x004: Reset.
    .word Default_Handler       // 0x008: NMI.
    .word Default_Handler       // 0x00C: HardFault.
    .word Default_Handler       // 0x010: MemManage.
    .word Default_Handler       // 0x014: BusFault.
    .word Default_Handler       // 0x018: UsageFault.
    .word Default_Handler       // 0x01C: reserved
    .word Default_Handler       // 0x020: reserved
    .word Default_Handler       // 0x024: reserved
    .word Default_Handler       // 0x028: reserved (ends @ 0x02B)
    .word Default_Handler       // 0x02C: SVCall.
    .word Default_Handler       // 0x030: DebugMon.
    .word Default_Handler       // 0x034: reserved.
    .word Default_Handler       // 0x038: PendSV.
    .word Default_Handler       // 0x03C: SysTick.
    .word Default_Handler       // 0x040: IRQ0 WWDG.
    .word Default_Handler       // 0x044: IRQ1 PVD.
    .word Default_Handler       // 0x048: IRQ2 TAMPER_STAMP.
    .word Default_Handler       // 0x04C: IRQ3 RTC_WKUP.
    .word Default_Handler       // 0x050: IRQ4 FLASH.
    .word Default_Handler       // 0x054: IRQ5 RCC.
    .word Default_Handler       // 0x058: IRQ6 EXTI0.
    .word Default_Handler       // 0x05C: IRQ7 EXTI1.
    .word Default_Handler       // 0x060: IRQ8 EXTI2.
    .word Default_Handler       // 0x064: IRQ9 EXTI3.
    .word Default_Handler       // 0x068: IRQ10 EXTI4.
    .word Default_Handler       // 0x06C: IRQ11 DMA1_Channel1.
    .word Default_Handler       // 0x070: IRQ12 DMA1_Channel2.
    .word Default_Handler       // 0x074: IRQ13 DMA1_Channel3.
    .word Default_Handler       // 0x078: IRQ14 DMA1_Channel4.
    .word Default_Handler       // 0x07C: IRQ15 DMA1_Channel5.
    .word Default_Handler       // 0x080: IRQ16 DMA1_Channel6.
    .word Default_Handler       // 0x084: IRQ17 DMA1_Channel7.
    .word Default_Handler       // 0x088: IRQ18 ADC1.
    .word Default_Handler       // 0x08C: IRQ19 USB_HP.
    .word Default_Handler       // 0x090: IRQ20 USB_LP.
    .word Default_Handler       // 0x094: IRQ21 DAC.
    .word Default_Handler       // 0x098: IRQ22 COMP_CA.
    .word Default_Handler       // 0x09C: IRQ23 EXTI9_5.
    .word Default_Handler       // 0x0A0: IRQ24 LCD.
    .word Default_Handler       // 0x0A4: IRQ25 TIM9.
    .word Default_Handler       // 0x0A8: IRQ26 TIM10.
    .word Default_Handler       // 0x0AC: IRQ27 TIM11.
    .word Default_Handler       // 0x0B0: IRQ28 TIM2.
    .word Default_Handler       // 0x0B4: IRQ29 TIM3.
    .word Default_Handler       // 0x0B8: IRQ30 TIM4.
    .word Default_Handler       // 0x0BC: IRQ31 I2C1_EV.
    .word Default_Handler       // 0x0C0: IRQ32 I2C1_ER.
    .word Default_Handler       // 0x0C4: IRQ33 I2C2_EV.
    .word Default_Handler       // 0x0C8: IRQ34 I2C2_ER.
    .word Default_Handler       // 0x0CC: IRQ35 SPI1.
    .word Default_Handler       // 0x0D0: IRQ36 SPI2.
    .word Default_Handler       // 0x0D4: IRQ37 USART1.
    .word Default_Handler       // 0x0D8: IRQ38 USART2.
    .word Default_Handler       // 0x0DC: IRQ39 USART3.
    .word Default_Handler       // 0x0E0: IRQ40 EXTI15_10.
    .word Default_Handler       // 0x0E4: IRQ41 RTC_Alarm.
    .word Default_Handler       // 0x0E8: IRQ42 USB_FS_WKUP.
    .word Default_Handler       // 0x0EC: IRQ43 TIM6.
    .word Default_Handler       // 0x0F0: IRQ44 TIM7.
    .word Default_Handler       // 0x0F4: IRQ45 SDIO.
    .word Default_Handler       // 0x0F8: IRQ46 TIM5.
    .word Default_Handler       // 0x0FC: IRQ47 SPI3.
    .word Default_Handler       // 0x100: IRQ48 UART4.
    .word Default_Handler       // 0x104: IRQ49 UART5.
    .word Default_Handler       // 0x108: IRQ50 DMA2_CH1.
    .word Default_Handler       // 0x10C: IRQ51 DMA2_CH2.
    .word Default_Handler       // 0x110: IRQ52 DMA2_CH3.
    .word Default_Handler       // 0x114: IRQ53 DMA2_CH4.
    .word Default_Handler       // 0x118: IRQ54 DMA2_CH5.
    .word Default_Handler       // 0x11C: IRQ55 AES.
    .word Default_Handler       // 0x120: IRQ56 COMP_ACQ.
    "#,
    estack = const ESTACK,
);