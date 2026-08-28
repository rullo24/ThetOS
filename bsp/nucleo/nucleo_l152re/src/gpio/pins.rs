use specs::bsp::gpio::Uninit;

use super::pin::Pin;
use super::port::PortA;

// user LED LD2 -> PA5 | Ref: ST UM1724 User manual 7.6 & 7.13 (Table 21)
pub const PA5: Pin<PortA, 5, Uninit> = Pin::new();
