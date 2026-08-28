// Ref: RM0038 2.3 (Table 5) memory map, 6.3.8 RCC_AHBENR
const GPIOB_BASE: usize = 0x4002_0400;
const GPIOB_RCC_BIT: u32 = 1;
define_port!(PortB, base = GPIOB_BASE, rcc_bit = GPIOB_RCC_BIT);
