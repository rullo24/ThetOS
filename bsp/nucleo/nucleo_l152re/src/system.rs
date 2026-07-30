
// core lib imports
use core::ptr::addr_of_mut;

// local imports
use crate::limits::MAX_TASKS;
use cortex_m::{V7mContextSwitch, CortexMCriticalSection, CortexMStackGuard};
use kernel::{Kernel, KernelStackResources};
use kernel::scheduler::FppScheduler;
use specs::common::TaskId;
use specs::kernel::{Result, TaskPriority, SystemTimer, TickAction};
use specs::arch::StackGuardContext;

/// global var to hold stack guard slots for static slot tables
static mut TASK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; MAX_TASKS] = [None; MAX_TASKS];

/// Board-facing system facade for Nucleo-L152RE.
pub struct System {
    kernel: Kernel<V7mContextSwitch, CortexMCriticalSection, FppScheduler, CortexMStackGuard, NullSystemTimer>,
}

impl System {

    /// DESCRIPTION
    /// create a board-composed system instance (`stack_pool` is SRAM reserved for kernel task stacks)
    pub fn new_with_pool(stack_pool: &'static mut [u8]) -> Self {
        Self {
            kernel: Kernel::new(
                V7mContextSwitch,
                CortexMCriticalSection,
                FppScheduler::new(),
                KernelStackResources::new(
                    stack_pool,
                    CortexMStackGuard,
                    unsafe { &mut *addr_of_mut!(TASK_STACK_GUARD_SLOTS) },
                ),
                NullSystemTimer,
            ),
        }
    }

    // TODO: remove this method before release
    /// DESCRIPTION
    /// request PendSV pending.
    pub fn request_pendsv_pending(&mut self) -> Result<()> {
        self.kernel.yield_now()
    }

    /// DESCRIPTION
    /// register a task with the system.
    pub fn spawn_task(
        &mut self,
        task_id: TaskId,
        priority: TaskPriority,
        stack_size: usize,
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Result<()> {
        return self.kernel.spawn_task(task_id, priority, stack_size, entry_point, entry_arg);
    }

    /// DESCRIPTION
    /// request a cooperative yield.
    pub fn yield_now(&mut self) -> Result<()> {
        self.kernel.yield_now()
    }

    /// DESCRIPTION
    /// start the system runtime.
    pub fn run(&self) -> ! {
        loop {
            core::hint::spin_loop();
        }

        // TODO: "hand over control to scheduler/context-switch start path

    }
}

/// DESCRIPTION
/// placeholder SystemTimer: always reports no tick action. keeps System
/// buildable ahead of the real SysTick-backed SystemTimer implementation
/// (Phase 3 bsp/mcu tickets); replace with the real impl once landed.
#[derive(Clone, Copy)]
pub struct NullSystemTimer;

impl SystemTimer for NullSystemTimer {
    type Error = core::convert::Infallible;

    fn initialise(&mut self, _reload_ticks: u32) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn start(&mut self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn stop(&mut self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn acknowledge_tick_interrupt(&mut self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn on_tick_interrupt(&mut self) -> core::result::Result<TickAction, Self::Error> {
        Ok(TickAction::None)
    }
}