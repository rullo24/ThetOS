// Ref: RM0038 2.3 (Table 5) memory map, 6.3.8 RCC_AHBENR
const GPIOH_BASE: usize = 0x4002_1400;
const GPIOH_RCC_BIT: u32 = 5;
define_port!(PortH, base = GPIOH_BASE, rcc_bit = GPIOH_RCC_BIT);
