// RCC AHB peripheral clock-enable register -> CLK required for GPIO usage
const RCC_BASE: usize = 0x4002_3800; // Ref: RM0038 2.3 (Table 5)
const RCC_AHBENR_OFFSET: usize = 0x1C; // Ref: RM0038 6.3.8
pub const RCC_AHBENR: *mut u32 = (RCC_BASE + RCC_AHBENR_OFFSET) as *mut u32;

// must explicitly enable RCC to power-up GPIO (RCC = Reset and Clock Control)
#[derive(Clone, Copy)]
pub struct RccEnable {
    pub reg: *mut u32,
    pub bit: u32,
}

// register locations for one GPIO port
pub trait GpioPort {
    const MODER: *mut u32; // pin direction
    const OTYPER: *mut u32; // output driver type
    const PUPDR: *mut u32; // internal resistor state
    const IDR: *const u32; // read-only, current input levels
    const BSRR: *mut u32; // atomic set/reset for pin writes
    const RCC_ENABLE: RccEnable;
}

// define a port whose block follows the standard STM32L1 GPIO layout | Ref: RM0038 7.4
macro_rules! define_port {
    ($name:ident, base = $base:expr, rcc_bit = $bit:expr) => {
        pub struct $name;
        impl $crate::gpio::port::GpioPort for $name {
            const MODER: *mut u32 = ($base + 0x00) as *mut u32;
            const OTYPER: *mut u32 = ($base + 0x04) as *mut u32;
            const PUPDR: *mut u32 = ($base + 0x0C) as *mut u32;
            const IDR: *const u32 = ($base + 0x10) as *const u32;
            const BSRR: *mut u32 = ($base + 0x18) as *mut u32;
            const RCC_ENABLE: $crate::gpio::port::RccEnable = $crate::gpio::port::RccEnable {
                reg: $crate::gpio::port::RCC_AHBENR,
                bit: $bit,
            };
        }
    };
}

mod port_a;
mod port_b;
mod port_c;
mod port_d;
mod port_e;
mod port_h;
pub use port_a::PortA;
pub use port_b::PortB;
pub use port_c::PortC;
pub use port_d::PortD;
pub use port_e::PortE;
pub use port_h::PortH;
