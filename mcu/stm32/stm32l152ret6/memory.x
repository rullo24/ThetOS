/* STM32L152RE (512K flash, 80K RAM) */

MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM   : ORIGIN = 0x20000000, LENGTH = 80K
}

__msp_stack_size = 0x4000; /* 16KB MSP stack size -> covers all ISR call-chain growth (SysTick/PendSV today, UART/GPIO IRQs later); RAM is 80K w/ ~7.5K used by .bss, so this is still cheap */