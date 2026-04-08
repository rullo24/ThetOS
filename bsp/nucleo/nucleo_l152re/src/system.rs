// core imports
use core::ptr::addr_of_mut;

// local imports
use cortex_m::V7mContextSwitch;
use kernel::Kernel;
use specs::arch::ContextSwitch;
use specs::common::{Result, TaskId};
use specs::kernel::{CriticalSection, SchedulerPolicy};

/// Board-facing system facade for Nucleo-L152RE.
pub struct System {
    _reserved: (),
}

impl System {

    /// DESCRIPTION
    /// create a board-composed system instance (`stack_pool` is SRAM reserved for kernel task stacks)
    pub fn new_with_pool(stack_pool: &'static mut [u8]) -> Self {
        Self {
            kernel: Kernel::new(
                V7mContextSwitch,
                NucleoCriticalSection,
                NucleoScheduler,
                stack_pool,
            ),
        }
    }

    /// DESCRIPTION
    /// request PendSV pending.
    pub fn request_pendsv_pending(&self) {
        V7mContextSwitch.trigger_pendsv_switch();
    }

    /// DESCRIPTION
    /// register a task with the system.
    pub fn spawn_task(
        &mut self,
        task_id: TaskId,
        stack_size: usize,
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Result<()> {
        return self.kernel.spawn_task(task_id, stack_size, entry_point, entry_arg);
    }

    /// DESCRIPTION
    /// request a cooperative yield.
    pub fn yield_now(&self) {
        self.kernel.yield_now();       
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