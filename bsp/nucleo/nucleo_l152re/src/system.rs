// core lib imports
use core::ptr::{addr_of_mut, null_mut};

// local imports
use crate::limits::MAX_TASKS;
use crate::system_timer::NucleoSystemTimer;
use cortex_m::v7m::{
    configure_kernel_interrupt_priorities, init_cycle_counter, read_cycle_counter,
    set_tick_callback,
};
use cortex_m::{wfi, CortexMCriticalSection, CortexMStackGuard, V7mContextSwitch};
use kernel::scheduler::FppScheduler;
use kernel::{Kernel, KernelStackResources};
use specs::arch::StackGuardContext;
use specs::common::TaskId;
use specs::kernel::{Result, SystemTimer, TaskPriority};

// RM0038 rev 18 s6.2.3 "MSI clock" pg 132: MSI raised to its fastest range (range 6) at boot
// by stm32l152ret6::clock::set_msi_max_range() -> SYSCLK = 4,194,304 Hz, not the 2,097,152 Hz default
const SYSCLK_HZ: u32 = 4_194_304;

// 100Hz -> a debug (unoptimized) build's on_tick_interrupt() measured ~7200 cycles, ~3.4x a 1ms budget on this MCU;
// 10ms gives comfortable headroom without needing a release build. See TICK_CYCLES_MAX below to re-check on this hardware.
const SYSTICK_PERIOD_MS: u32 = 10;

// reload = SYSCLK_HZ*period_ms/1000 - 1 (ARM N-1 formula); ~999.928us actual, negligible vs MSI's ~1% tolerance
const SYSTICK_RELOAD_TICKS: u32 = (SYSCLK_HZ * SYSTICK_PERIOD_MS) / 1000 - 1;

/// global var to hold stack guard slots for static slot tables
static mut TASK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; MAX_TASKS] = [None; MAX_TASKS];

/// dedicated stack for the idle task -> deliberately separate from the user-supplied stack_pool.
/// plain [u8; N] statics aren't guaranteed 8-byte (AAPCS) alignment, so this is forced explicitly.
const IDLE_STACK_SIZE_BYTES: usize = 512;
#[repr(align(8))]
struct AlignedIdleStack(#[allow(dead_code)] [u8; IDLE_STACK_SIZE_BYTES]); // only ever accessed via raw pointer arithmetic
static mut IDLE_STACK: AlignedIdleStack = AlignedIdleStack([0; IDLE_STACK_SIZE_BYTES]);

/// single running System instance -> reachable from the tick callback
static mut SYSTEM: Option<System> = None;

/// how many CPU cycles the most recent tick interrupt (on_tick_interrupt(), incl. reschedule()) took to run
#[no_mangle]
pub static mut TICK_CYCLES_LAST: u32 = 0;
/// worst-case cycle count seen for a tick interrupt since boot -> compare against SYSTICK_RELOAD_TICKS to check budget headroom
#[no_mangle]
pub static mut TICK_CYCLES_MAX: u32 = 0;

/// DESCRIPTION
/// tick callback registered with arch -> forwards the tick into the running System's kernel, timing it via the DWT cycle counter
fn on_systick_tick() {
    unsafe {
        if let Some(system) = (*addr_of_mut!(SYSTEM)).as_mut() {
            let start = read_cycle_counter();
            let _ = system.kernel.on_tick_interrupt();
            let elapsed = read_cycle_counter().wrapping_sub(start);
            TICK_CYCLES_LAST = elapsed;
            if elapsed > TICK_CYCLES_MAX {
                TICK_CYCLES_MAX = elapsed;
            }
        }
    }
}

/// DESCRIPTION
/// block the calling task until at least the given tick -> callable from task code.
pub fn block_current_task_until(wake_at_tick: u64) -> Result<()> {
    unsafe {
        (*addr_of_mut!(SYSTEM))
            .as_mut()
            .ok_or(specs::kernel::KernelError::InvalidState)?
            .block_current_task_until(wake_at_tick)
    }
}

/// DESCRIPTION
/// current tick count -> callable from task code, e.g. to compute a delay deadline.
pub fn current_tick() -> Result<u64> {
    unsafe {
        (*addr_of_mut!(SYSTEM))
            .as_mut()
            .ok_or(specs::kernel::KernelError::InvalidState)?
            .current_tick()
    }
}

/// DESCRIPTION
/// convert a millisecond duration to whole ticks, rounded up so the delay is never shorter than requested
fn ms_to_ticks(ms: u32) -> u64 {
    ((ms + SYSTICK_PERIOD_MS - 1) / SYSTICK_PERIOD_MS) as u64
}

/// DESCRIPTION
/// delay the calling task by at least `ms` milliseconds -> callable from task code.
pub fn delay_ms(ms: u32) -> Result<()> {
    let ticks = ms_to_ticks(ms);
    if ticks == 0 {
        return Ok(());
    }
    block_current_task_until(current_tick()? + ticks)
}

/// DESCRIPTION
/// idle task body -> lowest priority, never blocks, runs only when nothing else is ready
extern "C" fn idle_task_entry(_arg: *mut ()) -> ! {
    loop {
        wfi();
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
            init_cycle_counter(); // powers TICK_CYCLES_LAST/MAX below
        }

        let mut system_timer = NucleoSystemTimer::new();
        system_timer
            .initialise(SYSTICK_RELOAD_TICKS)
            .expect("SysTick initialise failed"); // configured but not started -> run() starts it once all init is complete

        let mut kernel = Kernel::new(
            V7mContextSwitch,
            CortexMCriticalSection,
            FppScheduler::new(),
            KernelStackResources::new(stack_pool, CortexMStackGuard, unsafe {
                &mut *addr_of_mut!(TASK_STACK_GUARD_SLOTS)
            }),
            system_timer,
        );

        // always present so block_current_task_until()/delay_ms() always have somewhere to
        // switch to -> without this, blocking the last runnable task would leave nothing to
        // run, so it's refused instead (see KernelError::NoRunnableTask); this task exists
        // so that refusal never happens in normal operation. Runs on its own dedicated stack,
        // not the user-supplied stack_pool.
        let idle_stack_limit = addr_of_mut!(IDLE_STACK) as *mut u8;
        let idle_stack_top = idle_stack_limit.wrapping_add(IDLE_STACK_SIZE_BYTES);
        kernel
            .spawn_idle_task(
                idle_task_entry,
                null_mut(),
                idle_stack_top,
                idle_stack_limit,
            )
            .expect("idle task spawn failed");

        Self { kernel }
    }

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
        if task_id == TaskId(0) {
            return Err(specs::kernel::KernelError::InvalidConfig); // reserved for the auto-spawned idle task
        }

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
    /// block the calling task until at least the given tick.
    pub fn block_current_task_until(&mut self, wake_at_tick: u64) -> Result<()> {
        self.kernel.block_current_task_until(wake_at_tick)
    }

    /// DESCRIPTION
    /// current tick count -> use to compute a wake_at_tick deadline.
    pub fn current_tick(&self) -> Result<u64> {
        self.kernel.current_tick()
    }

    /// DESCRIPTION
    /// delay the calling task by at least `ms` milliseconds.
    pub fn delay_ms(&mut self, ms: u32) -> Result<()> {
        let ticks = ms_to_ticks(ms);
        if ticks == 0 {
            return Ok(());
        }
        self.block_current_task_until(self.current_tick()? + ticks)
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
