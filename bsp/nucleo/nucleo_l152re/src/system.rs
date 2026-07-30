
// core lib imports
use core::ptr::addr_of_mut;

// local imports
use crate::limits::MAX_TASKS;
use crate::system_timer::NucleoSystemTimer;
use cortex_m::{V7mContextSwitch, CortexMCriticalSection, CortexMStackGuard};
use cortex_m::v7m::{configure_kernel_interrupt_priorities, set_tick_callback};
use kernel::{Kernel, KernelStackResources};
use kernel::scheduler::FppScheduler;
use specs::common::TaskId;
use specs::kernel::{Result, TaskPriority, SystemTimer};
use specs::arch::StackGuardContext;

// RM0038 rev 18 s6.2.3 "MSI clock" pg 132: SYSCLK default post-reset = MSI @ 2,097,152 Hz
const SYSCLK_HZ: u32 = 2_097_152;

// standard 1kHz system tick; future delay_ms() counts ticks for visible timing
const SYSTICK_PERIOD_MS: u32 = 1;

// reload = SYSCLK_HZ*period_ms/1000 - 1 (ARM N-1 formula); ~999.928us actual, negligible vs MSI's ~1% tolerance
const SYSTICK_RELOAD_TICKS: u32 = (SYSCLK_HZ * SYSTICK_PERIOD_MS) / 1000 - 1;

/// global var to hold stack guard slots for static slot tables
static mut TASK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; MAX_TASKS] = [None; MAX_TASKS];

/// single running System instance -> reachable from the tick callback, populated by install_as_tick_source()
static mut SYSTEM: Option<System> = None;

/// DESCRIPTION
/// tick callback registered with arch -> forwards the tick into the running System's kernel
fn on_systick_tick() {
    unsafe {
        if let Some(system) = (*addr_of_mut!(SYSTEM)).as_mut() {
            let _ = system.kernel.on_tick_interrupt();
        }
    }
}

/// Board-facing system facade for Nucleo-L152RE.
pub struct System {
    kernel: Kernel<V7mContextSwitch, CortexMCriticalSection, FppScheduler, CortexMStackGuard, NucleoSystemTimer>,
}

impl System {

    /// DESCRIPTION
    /// create a board-composed system instance (`stack_pool` is SRAM reserved for kernel task stacks)
    pub fn new_with_pool(stack_pool: &'static mut [u8]) -> Self {
        // PendSV/SysTick set to same lowest priority before the timer starts (standard Cortex-M RTOS convention)
        unsafe {
            configure_kernel_interrupt_priorities();
        }

        let mut system_timer = NucleoSystemTimer::new();
        system_timer
            .initialise(SYSTICK_RELOAD_TICKS)
            .expect("SysTick initialise failed");
        system_timer.start().expect("SysTick start failed");

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
                system_timer,
            ),
        }
    }

    /// DESCRIPTION
    /// move this System into the global static and register it as the SysTick tick source; caller must ensure this is called at most once
    pub unsafe fn install_as_tick_source(self) -> &'static mut System {
        unsafe {
            SYSTEM = Some(self);
            set_tick_callback(on_systick_tick);
            (*addr_of_mut!(SYSTEM)).as_mut().unwrap()
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