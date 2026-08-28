// Ref: RM0038 2.3 (Table 5) memory map, 6.3.8 RCC_AHBENR
const GPIOA_BASE: usize = 0x4002_0000;
const GPIOA_RCC_BIT: u32 = 0;
define_port!(PortA, base = GPIOA_BASE, rcc_bit = GPIOA_RCC_BIT);
