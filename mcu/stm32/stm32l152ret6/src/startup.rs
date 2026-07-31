/// Reset path, vector table, and RAM init for STM32L152RE (Cortex-M3).
use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

// pulled from linker.ld
const RAM_ORIGIN: u32 = 0x2000_0000;
const RAM_LENGTH_BYTES: u32 = 80 * 1024;
const ESTACK: u32 = RAM_ORIGIN + RAM_LENGTH_BYTES;
const FLASH_ORIGIN: u32 = 0x0800_0000;
const SCB_VTOR: *mut u32 = 0xE000_ED08 as *mut u32;

// defining linker symbols (match linker.ld)
unsafe extern "C" {
    // importing symbols like this is not memory-layout specific (this is not a struct)
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
        write_volatile(SCB_VTOR, FLASH_ORIGIN); // boot-time remap can alias 0x0 away from flash, so point VTOR at flash explicitly or exception vectors fetch the wrong table
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
    while (dst as usize) < (dst_end as usize) {
        // iterate over each RAM byte
        write_volatile(dst, read_volatile(src)); // write the byte from FLASH to RAM
        dst = dst.wrapping_add(1); // increment the RAM ptr
        src = src.wrapping_add(1); // increment the RAM ptr
    }
}

// zero the bss sections
unsafe fn zero_bss() {
    let mut p_bss_curr: *mut u8 = addr_of_mut!(__sbss); // start of bss section in RAM
    let end_bss: *mut u8 = addr_of_mut!(__ebss); // end of bss section in RAM
    while (p_bss_curr as usize) < (end_bss as usize) {
        // iterate over each bss byte
        write_volatile(p_bss_curr, 0); // write 0x0 to current byte
        p_bss_curr = p_bss_curr.wrapping_add(1); // increment ptr
    }
}

// embeds asm for the vector table in flash
// NOTE: mcu reference manual is Cat.5 (found in table 3 - pg 41/904)
// NOTE: vector table defined in STM32L152xx reference manual, table 51 (pg 235/904)
core::arch::global_asm!(
    r#"
    .section .vector_table,"a",%progbits        // put following bytes in section `.vector_table`, allocatable, program bits (normal flash data)
    .align 2                                    // align to 4 bytes (thumb code is 2-byte -> align 2*2 = 4 bytes (32-bit words))
    .word {estack}                              // 0x000: initial MSP.
    .word Reset                                 // 0x004: Reset.
    .word NMI_Handler                           // 0x008: NMI.
    .word HardFault_Handler                     // 0x00C: HardFault.
    .word MemManage_Handler                     // 0x010: MemManage.
    .word BusFault_Handler                      // 0x014: BusFault.
    .word UsageFault_Handler                    // 0x018: UsageFault.
    .word Default_Handler                       // 0x01C: reserved
    .word Default_Handler                       // 0x020: reserved
    .word Default_Handler                       // 0x024: reserved
    .word Default_Handler                       // 0x028: reserved (ends @ 0x02B)
    .word SVC_Handler                           // 0x02C: SVCall.
    .word DebugMon_Handler                      // 0x030: DebugMon.
    .word Default_Handler                       // 0x034: reserved.
    .word PendSV_Handler                        // 0x038: PendSV.
    .word SysTick_Handler                       // 0x03C: SysTick.
    .word WWDG_IRQHandler                       // 0x040: IRQ0 WWDG.
    .word PVD_IRQHandler                        // 0x044: IRQ1 PVD.
    .word TAMPER_STAMP_IRQHandler               // 0x048: IRQ2 TAMPER_STAMP.
    .word RTC_WKUP_IRQHandler                   // 0x04C: IRQ3 RTC_WKUP.
    .word FLASH_IRQHandler                      // 0x050: IRQ4 FLASH.
    .word RCC_IRQHandler                        // 0x054: IRQ5 RCC.
    .word EXTI0_IRQHandler                      // 0x058: IRQ6 EXTI0.
    .word EXTI1_IRQHandler                      // 0x05C: IRQ7 EXTI1.
    .word EXTI2_IRQHandler                      // 0x060: IRQ8 EXTI2.
    .word EXTI3_IRQHandler                      // 0x064: IRQ9 EXTI3.
    .word EXTI4_IRQHandler                      // 0x068: IRQ10 EXTI4.
    .word DMA1_Channel1_IRQHandler              // 0x06C: IRQ11 DMA1_Channel1.
    .word DMA1_Channel2_IRQHandler              // 0x070: IRQ12 DMA1_Channel2.
    .word DMA1_Channel3_IRQHandler              // 0x074: IRQ13 DMA1_Channel3.
    .word DMA1_Channel4_IRQHandler              // 0x078: IRQ14 DMA1_Channel4.
    .word DMA1_Channel5_IRQHandler              // 0x07C: IRQ15 DMA1_Channel5.
    .word DMA1_Channel6_IRQHandler              // 0x080: IRQ16 DMA1_Channel6.
    .word DMA1_Channel7_IRQHandler              // 0x084: IRQ17 DMA1_Channel7.
    .word ADC1_IRQHandler                       // 0x088: IRQ18 ADC1.
    .word USB_HP_IRQHandler                     // 0x08C: IRQ19 USB_HP.
    .word USB_LP_IRQHandler                     // 0x090: IRQ20 USB_LP.
    .word DAC_IRQHandler                        // 0x094: IRQ21 DAC.
    .word COMP_CA_IRQHandler                    // 0x098: IRQ22 COMP_CA.
    .word EXTI9_5_IRQHandler                    // 0x09C: IRQ23 EXTI9_5.
    .word LCD_IRQHandler                        // 0x0A0: IRQ24 LCD.
    .word TIM9_IRQHandler                       // 0x0A4: IRQ25 TIM9.
    .word TIM10_IRQHandler                      // 0x0A8: IRQ26 TIM10.
    .word TIM11_IRQHandler                      // 0x0AC: IRQ27 TIM11.
    .word TIM2_IRQHandler                       // 0x0B0: IRQ28 TIM2.
    .word TIM3_IRQHandler                       // 0x0B4: IRQ29 TIM3.
    .word TIM4_IRQHandler                       // 0x0B8: IRQ30 TIM4.
    .word I2C1_EV_IRQHandler                    // 0x0BC: IRQ31 I2C1_EV.
    .word I2C1_ER_IRQHandler                    // 0x0C0: IRQ32 I2C1_ER.
    .word I2C2_EV_IRQHandler                    // 0x0C4: IRQ33 I2C2_EV.
    .word I2C2_ER_IRQHandler                    // 0x0C8: IRQ34 I2C2_ER.
    .word SPI1_IRQHandler                       // 0x0CC: IRQ35 SPI1.
    .word SPI2_IRQHandler                       // 0x0D0: IRQ36 SPI2.
    .word USART1_IRQHandler                     // 0x0D4: IRQ37 USART1.
    .word USART2_IRQHandler                     // 0x0D8: IRQ38 USART2.
    .word USART3_IRQHandler                     // 0x0DC: IRQ39 USART3.
    .word EXTI15_10_IRQHandler                  // 0x0E0: IRQ40 EXTI15_10.
    .word RTC_Alarm_IRQHandler                  // 0x0E4: IRQ41 RTC_Alarm.
    .word USB_FS_WKUP_IRQHandler                // 0x0E8: IRQ42 USB_FS_WKUP.
    .word TIM6_IRQHandler                       // 0x0EC: IRQ43 TIM6.
    .word TIM7_IRQHandler                       // 0x0F0: IRQ44 TIM7.
    .word SDIO_IRQHandler                       // 0x0F4: IRQ45 SDIO.
    .word TIM5_IRQHandler                       // 0x0F8: IRQ46 TIM5.
    .word SPI3_IRQHandler                       // 0x0FC: IRQ47 SPI3.
    .word UART4_IRQHandler                      // 0x100: IRQ48 UART4.
    .word UART5_IRQHandler                      // 0x104: IRQ49 UART5.
    .word DMA2_Channel1_IRQHandler              // 0x108: IRQ50 DMA2_CH1.
    .word DMA2_Channel2_IRQHandler              // 0x10C: IRQ51 DMA2_CH2.
    .word DMA2_Channel3_IRQHandler              // 0x110: IRQ52 DMA2_CH3.
    .word DMA2_Channel4_IRQHandler              // 0x114: IRQ53 DMA2_CH4.
    .word DMA2_Channel5_IRQHandler              // 0x118: IRQ54 DMA2_CH5.
    .word AES_IRQHandler                        // 0x11C: IRQ55 AES.
    .word COMP_ACQ_IRQHandler                   // 0x120: IRQ56 COMP_ACQ.
    "#,
    estack = const ESTACK,
);

// weak alias every named vector to Default_Handler -> any crate can override by defining the same symbol name
core::arch::global_asm!(
    r#"
    .weak NMI_Handler
    .thumb_set NMI_Handler, Default_Handler
    .weak HardFault_Handler
    .thumb_set HardFault_Handler, Default_Handler
    .weak MemManage_Handler
    .thumb_set MemManage_Handler, Default_Handler
    .weak BusFault_Handler
    .thumb_set BusFault_Handler, Default_Handler
    .weak UsageFault_Handler
    .thumb_set UsageFault_Handler, Default_Handler
    .weak SVC_Handler
    .thumb_set SVC_Handler, Default_Handler
    .weak DebugMon_Handler
    .thumb_set DebugMon_Handler, Default_Handler
    .weak PendSV_Handler
    .thumb_set PendSV_Handler, Default_Handler
    .weak SysTick_Handler
    .thumb_set SysTick_Handler, Default_Handler

    .weak WWDG_IRQHandler
    .thumb_set WWDG_IRQHandler, Default_Handler
    .weak PVD_IRQHandler
    .thumb_set PVD_IRQHandler, Default_Handler
    .weak TAMPER_STAMP_IRQHandler
    .thumb_set TAMPER_STAMP_IRQHandler, Default_Handler
    .weak RTC_WKUP_IRQHandler
    .thumb_set RTC_WKUP_IRQHandler, Default_Handler
    .weak FLASH_IRQHandler
    .thumb_set FLASH_IRQHandler, Default_Handler
    .weak RCC_IRQHandler
    .thumb_set RCC_IRQHandler, Default_Handler
    .weak EXTI0_IRQHandler
    .thumb_set EXTI0_IRQHandler, Default_Handler
    .weak EXTI1_IRQHandler
    .thumb_set EXTI1_IRQHandler, Default_Handler
    .weak EXTI2_IRQHandler
    .thumb_set EXTI2_IRQHandler, Default_Handler
    .weak EXTI3_IRQHandler
    .thumb_set EXTI3_IRQHandler, Default_Handler
    .weak EXTI4_IRQHandler
    .thumb_set EXTI4_IRQHandler, Default_Handler
    .weak DMA1_Channel1_IRQHandler
    .thumb_set DMA1_Channel1_IRQHandler, Default_Handler
    .weak DMA1_Channel2_IRQHandler
    .thumb_set DMA1_Channel2_IRQHandler, Default_Handler
    .weak DMA1_Channel3_IRQHandler
    .thumb_set DMA1_Channel3_IRQHandler, Default_Handler
    .weak DMA1_Channel4_IRQHandler
    .thumb_set DMA1_Channel4_IRQHandler, Default_Handler
    .weak DMA1_Channel5_IRQHandler
    .thumb_set DMA1_Channel5_IRQHandler, Default_Handler
    .weak DMA1_Channel6_IRQHandler
    .thumb_set DMA1_Channel6_IRQHandler, Default_Handler
    .weak DMA1_Channel7_IRQHandler
    .thumb_set DMA1_Channel7_IRQHandler, Default_Handler
    .weak ADC1_IRQHandler
    .thumb_set ADC1_IRQHandler, Default_Handler
    .weak USB_HP_IRQHandler
    .thumb_set USB_HP_IRQHandler, Default_Handler
    .weak USB_LP_IRQHandler
    .thumb_set USB_LP_IRQHandler, Default_Handler
    .weak DAC_IRQHandler
    .thumb_set DAC_IRQHandler, Default_Handler
    .weak COMP_CA_IRQHandler
    .thumb_set COMP_CA_IRQHandler, Default_Handler
    .weak EXTI9_5_IRQHandler
    .thumb_set EXTI9_5_IRQHandler, Default_Handler
    .weak LCD_IRQHandler
    .thumb_set LCD_IRQHandler, Default_Handler
    .weak TIM9_IRQHandler
    .thumb_set TIM9_IRQHandler, Default_Handler
    .weak TIM10_IRQHandler
    .thumb_set TIM10_IRQHandler, Default_Handler
    .weak TIM11_IRQHandler
    .thumb_set TIM11_IRQHandler, Default_Handler
    .weak TIM2_IRQHandler
    .thumb_set TIM2_IRQHandler, Default_Handler
    .weak TIM3_IRQHandler
    .thumb_set TIM3_IRQHandler, Default_Handler
    .weak TIM4_IRQHandler
    .thumb_set TIM4_IRQHandler, Default_Handler
    .weak I2C1_EV_IRQHandler
    .thumb_set I2C1_EV_IRQHandler, Default_Handler
    .weak I2C1_ER_IRQHandler
    .thumb_set I2C1_ER_IRQHandler, Default_Handler
    .weak I2C2_EV_IRQHandler
    .thumb_set I2C2_EV_IRQHandler, Default_Handler
    .weak I2C2_ER_IRQHandler
    .thumb_set I2C2_ER_IRQHandler, Default_Handler
    .weak SPI1_IRQHandler
    .thumb_set SPI1_IRQHandler, Default_Handler
    .weak SPI2_IRQHandler
    .thumb_set SPI2_IRQHandler, Default_Handler
    .weak USART1_IRQHandler
    .thumb_set USART1_IRQHandler, Default_Handler
    .weak USART2_IRQHandler
    .thumb_set USART2_IRQHandler, Default_Handler
    .weak USART3_IRQHandler
    .thumb_set USART3_IRQHandler, Default_Handler
    .weak EXTI15_10_IRQHandler
    .thumb_set EXTI15_10_IRQHandler, Default_Handler
    .weak RTC_Alarm_IRQHandler
    .thumb_set RTC_Alarm_IRQHandler, Default_Handler
    .weak USB_FS_WKUP_IRQHandler
    .thumb_set USB_FS_WKUP_IRQHandler, Default_Handler
    .weak TIM6_IRQHandler
    .thumb_set TIM6_IRQHandler, Default_Handler
    .weak TIM7_IRQHandler
    .thumb_set TIM7_IRQHandler, Default_Handler
    .weak SDIO_IRQHandler
    .thumb_set SDIO_IRQHandler, Default_Handler
    .weak TIM5_IRQHandler
    .thumb_set TIM5_IRQHandler, Default_Handler
    .weak SPI3_IRQHandler
    .thumb_set SPI3_IRQHandler, Default_Handler
    .weak UART4_IRQHandler
    .thumb_set UART4_IRQHandler, Default_Handler
    .weak UART5_IRQHandler
    .thumb_set UART5_IRQHandler, Default_Handler
    .weak DMA2_Channel1_IRQHandler
    .thumb_set DMA2_Channel1_IRQHandler, Default_Handler
    .weak DMA2_Channel2_IRQHandler
    .thumb_set DMA2_Channel2_IRQHandler, Default_Handler
    .weak DMA2_Channel3_IRQHandler
    .thumb_set DMA2_Channel3_IRQHandler, Default_Handler
    .weak DMA2_Channel4_IRQHandler
    .thumb_set DMA2_Channel4_IRQHandler, Default_Handler
    .weak DMA2_Channel5_IRQHandler
    .thumb_set DMA2_Channel5_IRQHandler, Default_Handler
    .weak AES_IRQHandler
    .thumb_set AES_IRQHandler, Default_Handler
    .weak COMP_ACQ_IRQHandler
    .thumb_set COMP_ACQ_IRQHandler, Default_Handler
    "#,
);
