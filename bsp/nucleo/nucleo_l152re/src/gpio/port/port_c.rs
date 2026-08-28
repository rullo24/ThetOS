// Ref: RM0038 2.3 (Table 5) memory map, 6.3.8 RCC_AHBENR

const GPIOC_BASE: usize = 0x4002_0800;
const GPIOC_RCC_BIT: u32 = 2;
define_port!(PortC, base = GPIOC_BASE, rcc_bit = GPIOC_RCC_BIT);
