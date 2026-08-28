// Ref: RM0038 2.3 (Table 5) memory map, 6.3.8 RCC_AHBENR

const GPIOD_BASE: usize = 0x4002_0C00;
const GPIOD_RCC_BIT: u32 = 3;
define_port!(PortD, base = GPIOD_BASE, rcc_bit = GPIOD_RCC_BIT);
