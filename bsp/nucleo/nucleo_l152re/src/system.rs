
// core lib imports
use core::ptr::addr_of_mut;

// local imports
use crate::limits::MAX_TASKS;
use cortex_m::{V7mContextSwitch, CortexMCriticalSection, CortexMStackGuard};
use kernel::{Kernel, KernelStackResources};
use kernel::scheduler::FppScheduler;
use specs::common::TaskId;
use specs::kernel::Result;
use specs::arch::StackGuardContext;

/// global var to hold stack guard slots for static slot tables
static mut TASK_STACK_GUARD_SLOTS: [Option<StackGuardContext>; MAX_TASKS] = [None; MAX_TASKS];

/// Board-facing system facade for Nucleo-L152RE.
pub struct System {
    kernel: Kernel<V7mContextSwitch, CortexMCriticalSection, FppScheduler, CortexMStackGuard>,
}

impl System {

    /// DESCRIPTION
    /// create a board-composed system instance (`stack_pool` is SRAM reserved for kernel task stacks)
    pub fn new_with_pool(stack_pool: &'static mut [u8]) -> Self {
        Self {
            kernel: Kernel::new(
                V7mContextSwitch,
                CortexMCriticalSection,
                FppScheduler,
                KernelStackResources::new(
                    stack_pool, 
                    CortexMStackGuard,
                    unsafe { &mut *addr_of_mut!(TASK_STACK_GUARD_SLOTS) },
                ),
            ),
        }
    }

    // TODO: remove this method before release
    /// DESCRIPTION
    /// request PendSV pending.
    pub fn request_pendsv_pending(&mut self) {
        self.kernel.yield_now();
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
    pub fn yield_now(&mut self) {
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