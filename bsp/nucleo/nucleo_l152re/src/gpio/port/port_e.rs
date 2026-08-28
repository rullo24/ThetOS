// Ref: RM0038 2.3 (Table 5) memory map, 6.3.8 RCC_AHBENR

const GPIOE_BASE: usize = 0x4002_1000;
const GPIOE_RCC_BIT: u32 = 4;
define_port!(PortE, base = GPIOE_BASE, rcc_bit = GPIOE_RCC_BIT);
