mod sealed {
    pub trait Sealed {}
}

// typestate marker
pub trait PwmMode: sealed::Sealed {}

pub struct Uninit; // timebase not configured yet
pub struct Active;

impl sealed::Sealed for Uninit {}
impl sealed::Sealed for Active {}
impl PwmMode for Uninit {}
impl PwmMode for Active {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PwmConfig {
    pub freq_hz: u32, // shared by every channel on the timer, fixed at into_active()
}

impl Default for PwmConfig {
    /// DESCRIPTION
    /// 10 kHz, a common motor-drive PWM frequency
    fn default() -> Self {
        Self { freq_hz: 10_000 }
    }
}

pub trait UninitPwm {
    // the running timebase; the board crate adds the per-channel accessors that consume pins
    type Active;

    /// DESCRIPTION
    /// program the timer period from `config` and return the running timebase
    fn into_active(self, config: PwmConfig) -> Self::Active;
}

pub trait PwmChannel {
    /// DESCRIPTION
    /// set the on-time as a fraction of the period; u16::MAX is full-on, resolution is the timer's period count
    fn set_duty(&mut self, duty: u16);

    /// DESCRIPTION
    /// drive the output pin from the timer
    fn enable(&mut self);

    /// DESCRIPTION
    /// stop driving the output pin; the timer keeps running for the other channels
    fn disable(&mut self);
}
