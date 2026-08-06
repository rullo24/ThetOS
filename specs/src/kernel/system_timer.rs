use core::fmt::Debug;
use core::result::Result;

/// outcome from handling a tick timer interrupt.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TickAction {
    None,
    RequestReschedule,
}

/// kernel-facing timer contract for tick to reschedule tasks
pub trait SystemTimer {
    type Error: Debug; /// error type for timer operations.

    /// DESCRIPTION
    /// configure timer cadence in hardware ticks
    fn initialise(&mut self, reload_ticks: u32) -> Result<(), Self::Error>;

    /// DESCRIPTION
    /// start periodic tick generation
    fn start(&mut self) -> Result<(), Self::Error>;

    /// DESCRIPTION
    /// stop periodic tick generation
    fn stop(&mut self) -> Result<(), Self::Error>;

    /// DESCRIPTION
    /// restart the current tick period from the top, without changing its configured cadence ->
    /// used when a task yields early, so the task it hands off to gets a full fresh period
    /// instead of being cut short by a tick that was already partway elapsed
    fn restart(&mut self) -> Result<(), Self::Error>;

    /// DESCRIPTION
    /// clear pending interrupt state after tick
    fn acknowledge_tick_interrupt(&mut self) -> Result<(), Self::Error>;

    /// DESCRIPTION
    /// handle timer interrupt and report scheduler action
    fn on_tick_interrupt(&mut self) -> Result<TickAction, Self::Error>;
    
}