// TIM2 / TIM3 edge-aligned PWM (RM0038 rev 18 ch.17 "General-purpose timers")
// one timer = one frequency, up to 4 independent-duty channels
use core::marker::PhantomData;
use core::ptr::{read_volatile, write_volatile};

use crate::clock::PCLK1_HZ;
use crate::gpio::Uninit as PinUninit;
use crate::gpio::{Pin, PortA};

// board-facing PWM contract + typestate markers, re-exported so a consumer never names `specs`
pub use specs::bsp::pwm::{Active, PwmChannel, PwmConfig, PwmMode, Uninit, UninitPwm};

// ---- RCC APB1 peripheral clock enable (RM0038 rev 18 s6.3.10) ----
const RCC_APB1ENR: *mut u32 = 0x4002_3824 as *mut u32; // RCC base 0x4002_3800 + 0x24 offset

// ---- TIM register offsets, common to TIM2..TIM5 (RM0038 rev 18 s17.4.21 register map) ----
const OFF_CR1: usize = 0x00;
const OFF_EGR: usize = 0x14;
const OFF_CCMR1: usize = 0x18;
const OFF_CCER: usize = 0x20;
const OFF_PSC: usize = 0x28;
const OFF_ARR: usize = 0x2C;
const OFF_CCR1: usize = 0x34;
const OFF_CCR2: usize = 0x38;

// ---- CR1 bits (RM0038 rev 18 s17.4.1) ----
const CR1_CEN: u32 = 1 << 0; // counter enable
const CR1_ARPE: u32 = 1 << 7; // auto-reload preload enable

// ---- EGR bits (RM0038 rev 18 s17.4.6) ----
const EGR_UG: u32 = 1 << 0; // update generation -> latch PSC / ARR / CCMR preloads

// ---- CCMR1 bits, output-compare layout (RM0038 rev 18 s17.4.7) ----
const CCMR1_OC1PE: u32 = 1 << 3; // CH1 preload enable
const CCMR1_OC1M_PWM1: u32 = 0b110 << 4; // CH1 mode = PWM mode 1
const CCMR1_OC2PE: u32 = 1 << 11; // CH2 preload enable
const CCMR1_OC2M_PWM1: u32 = 0b110 << 12; // CH2 mode = PWM mode 1

// ---- CCER bits (RM0038 rev 18 s17.4.9) ----
const CCER_CC1E: u32 = 1 << 0; // CH1 output enable
const CCER_CC2E: u32 = 1 << 4; // CH2 output enable

// alternate function for the timer pins (STM32L152xE datasheet, alternate function table)
const TIM2_AF: u8 = 1;
const TIM3_AF: u8 = 2;

/// DESCRIPTION
/// register locations + APB1 clock bit for one general-purpose timer
pub trait PwmTimer {
    const BASE: usize;
    const RCC_BIT: u32; // position in RCC_APB1ENR

    const CR1: *mut u32 = (Self::BASE + OFF_CR1) as *mut u32;
    const EGR: *mut u32 = (Self::BASE + OFF_EGR) as *mut u32;
    const CCMR1: *mut u32 = (Self::BASE + OFF_CCMR1) as *mut u32;
    const CCER: *mut u32 = (Self::BASE + OFF_CCER) as *mut u32;
    const PSC: *mut u32 = (Self::BASE + OFF_PSC) as *mut u32;
    const ARR: *mut u32 = (Self::BASE + OFF_ARR) as *mut u32;
    const CCR1: *mut u32 = (Self::BASE + OFF_CCR1) as *mut u32;
    const CCR2: *mut u32 = (Self::BASE + OFF_CCR2) as *mut u32;
}

pub struct Tim2;
pub struct Tim3;

impl PwmTimer for Tim2 {
    const BASE: usize = 0x4000_0000;
    const RCC_BIT: u32 = 0; // TIM2EN
}

impl PwmTimer for Tim3 {
    const BASE: usize = 0x4000_0400;
    const RCC_BIT: u32 = 1; // TIM3EN
}

/// DESCRIPTION
/// PWM timebase handle -> MODE is Uninit until into_active() programs the period
pub struct Pwm<TIM: PwmTimer, MODE: PwmMode> {
    _marker: PhantomData<(TIM, MODE)>,
}

impl<TIM: PwmTimer> Pwm<TIM, Uninit> {
    /// DESCRIPTION
    /// unconfigured handle for this timer -> does not touch hardware
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<TIM: PwmTimer> Default for Pwm<TIM, Uninit> {
    /// DESCRIPTION
    /// same as new() -> an unconfigured timer handle
    fn default() -> Self {
        Self::new()
    }
}

impl<TIM: PwmTimer> UninitPwm for Pwm<TIM, Uninit> {
    type Active = Pwm<TIM, Active>;

    /// DESCRIPTION
    /// enable the timer clock, program PSC / ARR for `freq_hz`, start the counter with both channels idle
    fn into_active(self, config: PwmConfig) -> Pwm<TIM, Active> {
        debug_assert!(config.freq_hz > 0, "PWM frequency must be non-zero");

        // total timer ticks per PWM period, then the smallest prescaler that keeps ARR in 16 bits
        let ticks = PCLK1_HZ / config.freq_hz;
        let psc = ticks.saturating_sub(1) / 65_536;
        let arr = (ticks / (psc + 1)).saturating_sub(1);

        unsafe {
            write_volatile(RCC_APB1ENR, read_volatile(RCC_APB1ENR) | (1 << TIM::RCC_BIT));

            write_volatile(TIM::PSC, psc);
            write_volatile(TIM::ARR, arr);
            write_volatile(
                TIM::CCMR1,
                CCMR1_OC1M_PWM1 | CCMR1_OC1PE | CCMR1_OC2M_PWM1 | CCMR1_OC2PE,
            );

            write_volatile(TIM::CR1, CR1_ARPE); // preload on, counter still stopped
            write_volatile(TIM::EGR, EGR_UG); // latch PSC / ARR / CCMR
            write_volatile(TIM::CR1, CR1_ARPE | CR1_CEN); // run
        }

        Pwm {
            _marker: PhantomData,
        }
    }
}

/// DESCRIPTION
/// one compare channel of a running timer -> starts disabled at 0% duty
pub struct Channel<TIM: PwmTimer> {
    ccr: *mut u32,
    ccer_mask: u32,
    _marker: PhantomData<TIM>,
}

impl<TIM: PwmTimer> Channel<TIM> {
    fn new(ccr: *mut u32, ccer_mask: u32) -> Self {
        Self {
            ccr,
            ccer_mask,
            _marker: PhantomData,
        }
    }
}

impl Pwm<Tim3, Active> {
    /// DESCRIPTION
    /// TIM3_CH1 on PA6 -> consumes the pin so it can't also be a GPIO
    pub fn channel1(&self, pin: Pin<PortA, 6, PinUninit>) -> Channel<Tim3> {
        let _routed = pin.into_alternate(TIM3_AF);
        Channel::new(Tim3::CCR1, CCER_CC1E)
    }

    /// DESCRIPTION
    /// TIM3_CH2 on PA7
    pub fn channel2(&self, pin: Pin<PortA, 7, PinUninit>) -> Channel<Tim3> {
        let _routed = pin.into_alternate(TIM3_AF);
        Channel::new(Tim3::CCR2, CCER_CC2E)
    }
}

impl Pwm<Tim2, Active> {
    /// DESCRIPTION
    /// TIM2_CH1 on PA5 (LD2) -> handy for a no-extra-hardware brightness sweep
    pub fn channel1(&self, pin: Pin<PortA, 5, PinUninit>) -> Channel<Tim2> {
        let _routed = pin.into_alternate(TIM2_AF);
        Channel::new(Tim2::CCR1, CCER_CC1E)
    }
}

impl<TIM: PwmTimer> PwmChannel for Channel<TIM> {
    /// DESCRIPTION
    /// u16::MAX maps to the full period; scaled against the live ARR
    fn set_duty(&mut self, duty: u16) {
        let period = unsafe { read_volatile(TIM::ARR) } + 1;
        let compare = (duty as u32 * period) / 65_536;
        unsafe { write_volatile(self.ccr, compare) };
    }

    /// DESCRIPTION
    /// connect this channel's compare output to its pin
    fn enable(&mut self) {
        unsafe { write_volatile(TIM::CCER, read_volatile(TIM::CCER) | self.ccer_mask) };
    }

    /// DESCRIPTION
    /// disconnect this channel's output; the timer keeps running for the others
    fn disable(&mut self) {
        unsafe { write_volatile(TIM::CCER, read_volatile(TIM::CCER) & !self.ccer_mask) };
    }
}
