// Board pin handles for the Nucleo-L152RE.
// Covers every GPIO on the STM32L152RE LQFP64 package: ports A/B/C in full, PD2, PH0/PH1.
// All are broken out on the CN7/CN10 morpho headers (a subset also on the Arduino headers).
// Each is an Uninit handle -> call `.into_output(..)` / `.into_input(..)` before use.
// Refs: STM32L152xE datasheet (pinouts), ST UM1724 (Nucleo-64 connectors / LEDs / button).

use specs::bsp::gpio::Uninit;

use super::pin::Pin;
use super::port::{PortA, PortB, PortC, PortD, PortH};

// ---- Port A ----
pub const PA0: Pin<PortA, 0, Uninit> = Pin::new();
pub const PA1: Pin<PortA, 1, Uninit> = Pin::new();
pub const PA2: Pin<PortA, 2, Uninit> = Pin::new(); // USART2_TX -> ST-LINK VCP by default
pub const PA3: Pin<PortA, 3, Uninit> = Pin::new(); // USART2_RX -> ST-LINK VCP by default
pub const PA4: Pin<PortA, 4, Uninit> = Pin::new();
pub const PA5: Pin<PortA, 5, Uninit> = Pin::new(); // user LED LD2 (green), active-high
pub const PA6: Pin<PortA, 6, Uninit> = Pin::new();
pub const PA7: Pin<PortA, 7, Uninit> = Pin::new();
pub const PA8: Pin<PortA, 8, Uninit> = Pin::new();
pub const PA9: Pin<PortA, 9, Uninit> = Pin::new();
pub const PA10: Pin<PortA, 10, Uninit> = Pin::new();
pub const PA11: Pin<PortA, 11, Uninit> = Pin::new();
pub const PA12: Pin<PortA, 12, Uninit> = Pin::new();
pub const PA13: Pin<PortA, 13, Uninit> = Pin::new(); // SWDIO -> reconfiguring breaks debug/flash
pub const PA14: Pin<PortA, 14, Uninit> = Pin::new(); // SWCLK -> reconfiguring breaks debug/flash
pub const PA15: Pin<PortA, 15, Uninit> = Pin::new();

// ---- Port B ----
pub const PB0: Pin<PortB, 0, Uninit> = Pin::new();
pub const PB1: Pin<PortB, 1, Uninit> = Pin::new();
pub const PB2: Pin<PortB, 2, Uninit> = Pin::new();
pub const PB3: Pin<PortB, 3, Uninit> = Pin::new(); // SWO / TRACESWO (optional debug trace)
pub const PB4: Pin<PortB, 4, Uninit> = Pin::new();
pub const PB5: Pin<PortB, 5, Uninit> = Pin::new();
pub const PB6: Pin<PortB, 6, Uninit> = Pin::new();
pub const PB7: Pin<PortB, 7, Uninit> = Pin::new();
pub const PB8: Pin<PortB, 8, Uninit> = Pin::new();
pub const PB9: Pin<PortB, 9, Uninit> = Pin::new();
pub const PB10: Pin<PortB, 10, Uninit> = Pin::new();
pub const PB11: Pin<PortB, 11, Uninit> = Pin::new();
pub const PB12: Pin<PortB, 12, Uninit> = Pin::new();
pub const PB13: Pin<PortB, 13, Uninit> = Pin::new();
pub const PB14: Pin<PortB, 14, Uninit> = Pin::new();
pub const PB15: Pin<PortB, 15, Uninit> = Pin::new();

// ---- Port C ----
pub const PC0: Pin<PortC, 0, Uninit> = Pin::new();
pub const PC1: Pin<PortC, 1, Uninit> = Pin::new();
pub const PC2: Pin<PortC, 2, Uninit> = Pin::new();
pub const PC3: Pin<PortC, 3, Uninit> = Pin::new();
pub const PC4: Pin<PortC, 4, Uninit> = Pin::new();
pub const PC5: Pin<PortC, 5, Uninit> = Pin::new();
pub const PC6: Pin<PortC, 6, Uninit> = Pin::new();
pub const PC7: Pin<PortC, 7, Uninit> = Pin::new();
pub const PC8: Pin<PortC, 8, Uninit> = Pin::new();
pub const PC9: Pin<PortC, 9, Uninit> = Pin::new();
pub const PC10: Pin<PortC, 10, Uninit> = Pin::new();
pub const PC11: Pin<PortC, 11, Uninit> = Pin::new();
pub const PC12: Pin<PortC, 12, Uninit> = Pin::new();
pub const PC13: Pin<PortC, 13, Uninit> = Pin::new(); // B1 user button (blue), active-low
pub const PC14: Pin<PortC, 14, Uninit> = Pin::new(); // OSC32_IN (LSE) -> free only if LSE unfitted
pub const PC15: Pin<PortC, 15, Uninit> = Pin::new(); // OSC32_OUT (LSE) -> free only if LSE unfitted

// ---- Port D ----
pub const PD2: Pin<PortD, 2, Uninit> = Pin::new(); // only Port D pin bonded on LQFP64

// ---- Port H ----
pub const PH0: Pin<PortH, 0, Uninit> = Pin::new(); // OSC_IN (HSE) -> check board solder bridges
pub const PH1: Pin<PortH, 1, Uninit> = Pin::new(); // OSC_OUT (HSE) -> check board solder bridges
