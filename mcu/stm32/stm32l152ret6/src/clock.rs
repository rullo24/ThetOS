/// MSI (Multi-Speed Internal) clock range configuration for STM32L152RE.
/// NOTE: mcu reference manual is Cat.5 (RM0038 rev 18), s6.2.3 "MSI clock" pg 132/904, RCC_ICSCR pg 145/904.
use core::ptr::{read_volatile, write_volatile};

const RCC_ICSCR: *mut u32 = 0x4002_3804 as *mut u32;

const ICSCR_MSIRANGE_SHIFT: u32 = 13;
const ICSCR_MSIRANGE_MASK: u32 = 0b111 << ICSCR_MSIRANGE_SHIFT;

/// MSI range 6: 4.194304 MHz -> the fastest of the 7 selectable MSI ranges (range 5, 2.097152 MHz, is the post-reset default)
const MSIRANGE_6_4_194_MHZ: u32 = 0b110 << ICSCR_MSIRANGE_SHIFT;

/// DESCRIPTION
/// raise the MSI clock from its post-reset default (range 5, ~2.097 MHz) to the fastest
/// available range (range 6, ~4.194 MHz). MSI is a direct RC range-select, not a PLL/crystal
/// that needs a ready handshake before switching -> RM0038's MSI clock description (s6.2.3)
/// only calls for writing MSIRANGE[2:0] in RCC_ICSCR, nothing more.
pub unsafe fn set_msi_max_range() {
    let icscr = read_volatile(RCC_ICSCR) & !ICSCR_MSIRANGE_MASK;
    write_volatile(RCC_ICSCR, icscr | MSIRANGE_6_4_194_MHZ);
}
