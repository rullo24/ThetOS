use core::result::Result;

/// low-level MCU timer capability contract (no board-facing behaviour).
pub trait TimerHardware {
    type Error: core::fmt::Debug;

    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn set_reload(&mut self, ticks: u32) -> Result<(), Self::Error>;
    fn current_tick(&self) -> Result<u64, Self::Error>;
    fn enable_interrupt(&mut self) -> Result<(), Self::Error>;
    fn disable_interrupt(&mut self) -> Result<(), Self::Error>;
    fn clear_pending(&mut self) -> Result<(), Self::Error>;
}
