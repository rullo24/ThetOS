// RM0038 rev 18 s6.2.3 "MSI clock" pg 132: MSI raised to its fastest range (range 6) at boot
// by stm32l152ret6::clock::set_msi_max_range() -> SYSCLK = 4,194,304 Hz, not the 2,097,152 Hz default
pub const SYSCLK_HZ: u32 = 4_194_304; // core CLK and SysTick reference
pub const PCLK1_HZ: u32 = SYSCLK_HZ; // APB1 peripheral CLK
