use core::result::Result;

/// DESCRIPTION
/// low-level CPU-core timer contract (Cortex-M SysTick)
pub trait SystemTicker {
    type Error: core::fmt::Debug;

    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn set_reload(&mut self, ticks: u32) -> Result<(), Self::Error>;

    /// restart the current countdown from the top, using whatever reload value is already set
    fn reset_counter(&mut self) -> Result<(), Self::Error>;

    fn current_tick(&self) -> Result<u64, Self::Error>;
    fn enable_interrupt(&mut self) -> Result<(), Self::Error>;
    fn disable_interrupt(&mut self) -> Result<(), Self::Error>;
    fn clear_pending(&mut self) -> Result<(), Self::Error>;
}
