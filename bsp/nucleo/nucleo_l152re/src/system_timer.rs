use cortex_m::{SysTickError, V7mSysTick};
use specs::arch::SystemTicker;
use specs::kernel::{SystemTimer, TickAction};

/// DESCRIPTION
/// bsp-composed SystemTimer -> wraps V7mSysTick, decides tick policy for this board
pub struct NucleoSystemTimer {
    systick: V7mSysTick,
}

impl NucleoSystemTimer {
    /// DESCRIPTION
    /// create a new bsp SystemTimer wrapping the SysTick peripheral handle
    pub const fn new() -> Self {
        Self { systick: V7mSysTick }
    }
}

impl SystemTimer for NucleoSystemTimer {
    type Error = SysTickError;

    /// DESCRIPTION
    /// configure the reload value and enable the SysTick interrupt (does not start the counter)
    fn initialise(&mut self, reload_ticks: u32) -> Result<(), Self::Error> {
        self.systick.set_reload(reload_ticks)?;
        self.systick.enable_interrupt()?;
        Ok(())
    }

    /// DESCRIPTION
    /// start the SysTick counter
    fn start(&mut self) -> Result<(), Self::Error> {
        self.systick.start()
    }

    /// DESCRIPTION
    /// stop the SysTick counter
    fn stop(&mut self) -> Result<(), Self::Error> {
        self.systick.stop()
    }

    /// DESCRIPTION
    /// clear the pending SysTick exception request
    fn acknowledge_tick_interrupt(&mut self) -> Result<(), Self::Error> {
        self.systick.clear_pending()
    }

    /// DESCRIPTION
    /// every tick requests a reschedule -> simplest correct policy for Gatekeeper 3's round-robin demo
    fn on_tick_interrupt(&mut self) -> Result<TickAction, Self::Error> {
        Ok(TickAction::RequestReschedule)
    }
}
