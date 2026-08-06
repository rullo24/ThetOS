// core lib imports
use core::ptr::addr_of_mut;

// local imports
use crate::limits::MAX_TASKS;
use crate::system_timer::NucleoSystemTimer;
use cortex_m::v7m::{configure_kernel_interrupt_priorities, set_tick_callback};
use cortex_m::{CortexMCriticalSection, CortexMStackGuard, V7mContextSwitch};
use kernel::scheduler::FppScheduler;
use kernel::{Kernel, KernelStackResources};
use specs::arch::StackGuardContext;
use specs::common::TaskId;
use specs::kernel::{Result, SystemTimer, TaskPriority};

// RM0038 rev 18 s6.2.3 "MSI clock" pg 132: SYSCLK default post-reset = MSI @ 2,097,152 Hz
const SYSCLK_HZ: u32 = 2_097_152;
const SYSTICK_PERIOD_MS: u32 = 1; // TEMP DIAGNOSTIC (#41): isolation test -> back to 1ms with the critical-section fix still in place, to see if the fix alone is sufficient

// reload = SYSCLK_HZ*period_ms/1000 - 1 (ARM N-1 formula); ~999.928us actual, negligible vs MSI's ~1% tolerance
const SYSTICK_RELOAD_TICKS: u32 = (SYSCLK_HZ * SYSTICK_PERIOD_MS) / 1000 - 1;

/// global var to hold stack guard slots for static slot tables
static mut TASK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; MAX_TASKS] = [None; MAX_TASKS];

/// single running System instance -> reachable from the tick callback
static mut SYSTEM: Option<System> = None;

// TEMP DIAGNOSTIC (#41): measures how many CPU cycles on_tick_interrupt() (incl. reschedule()) actually takes, vs the SysTick tick budget
#[no_mangle]
pub static mut DIAG_TICK_CYCLES_LAST: u32 = 0;
#[no_mangle]
pub static mut DIAG_TICK_CYCLES_MAX: u32 = 0;

/// DESCRIPTION
/// tick callback registered with arch -> forwards the tick into the running System's kernel
fn on_systick_tick() {
    unsafe {
        if let Some(system) = (*addr_of_mut!(SYSTEM)).as_mut() {
            let start = cortex_m::v7m::read_cycle_counter(); // TEMP DIAGNOSTIC (#41)
            let _ = system.kernel.on_tick_interrupt();
            let elapsed = cortex_m::v7m::read_cycle_counter().wrapping_sub(start); // TEMP DIAGNOSTIC (#41)
            DIAG_TICK_CYCLES_LAST = elapsed;
            if elapsed > DIAG_TICK_CYCLES_MAX {
                DIAG_TICK_CYCLES_MAX = elapsed;
            }
        }
    }
}

/// TEMP DIAGNOSTIC (#41): lets a running task trigger a yield from Thread-mode code directly, bypassing SysTick entirely
pub fn diag_yield_from_task() {
    unsafe {
        if let Some(system) = (*addr_of_mut!(SYSTEM)).as_mut() {
            let _ = system.kernel.yield_now();
        }
    }
}

/// TEMP DIAGNOSTIC (#41): minimal, reschedule()-bypassing switch straight to a given task
pub fn diag_minimal_switch_to(task_id: TaskId) {
    unsafe {
        if let Some(system) = (*addr_of_mut!(SYSTEM)).as_mut() {
            let _ = system.kernel.diag_minimal_switch_to(task_id);
        }
    }
}

/// Board-facing system facade for Nucleo-L152RE.
pub struct System {
    kernel: Kernel<
        V7mContextSwitch,
        CortexMCriticalSection,
        FppScheduler,
        CortexMStackGuard,
        NucleoSystemTimer,
    >,
}

impl System {
    /// DESCRIPTION
    /// create a board-composed system instance (`stack_pool` is SRAM reserved for kernel task stacks)
    pub fn new_with_pool(stack_pool: &'static mut [u8]) -> Self {
        // PendSV/SysTick set to same lowest priority before the timer starts (standard Cortex-M RTOS convention)
        unsafe {
            configure_kernel_interrupt_priorities();
            cortex_m::v7m::init_cycle_counter(); // TEMP DIAGNOSTIC (#41)
        }

        let mut system_timer = NucleoSystemTimer::new();
        system_timer
            .initialise(SYSTICK_RELOAD_TICKS)
            .expect("SysTick initialise failed"); // configured but not started -> run() starts it once all init is complete

        Self {
            kernel: Kernel::new(
                V7mContextSwitch,
                CortexMCriticalSection,
                FppScheduler::new(),
                KernelStackResources::new(stack_pool, CortexMStackGuard, unsafe {
                    &mut *addr_of_mut!(TASK_STACK_GUARD_SLOTS)
                }),
                system_timer,
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
        return self
            .kernel
            .spawn_task(task_id, priority, stack_size, entry_point, entry_arg);
    }

    /// DESCRIPTION
    /// request a cooperative yield.
    pub fn yield_now(&mut self) -> Result<()> {
        self.kernel.yield_now()
    }

    /// DESCRIPTION
    /// start the system runtime. call once, after all tasks are spawned -> installs the tick source and starts SysTick, so no tick can land before init is complete. consumes `self` because this never returns.
    pub fn run(self) -> ! {
        // set SysTick callback
        let system: &'static mut System = unsafe {
            SYSTEM = Some(self);
            set_tick_callback(on_systick_tick);
            (*addr_of_mut!(SYSTEM)).as_mut().unwrap()
        };

        // start SysTick and dispatch into the first task (initialisation now complete)
        system.kernel.start().expect("kernel start failed");

        loop {
            core::hint::spin_loop();
        }
    }
}
