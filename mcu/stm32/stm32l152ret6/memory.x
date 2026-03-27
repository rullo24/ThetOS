/* STM32L152RE (512K flash, 80K RAM) */

MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM   : ORIGIN = 0x20000000, LENGTH = 80K
}

__msp_stack_size = 0x1000; /* 4KB MSP stack size -> override 2KB default */