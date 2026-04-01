use specs::common::{Result, TaskId};

/// Board-facing system facade for Nucleo-L152RE.
pub struct System {
    _reserved: (),

    // TODO: "internal composed kernel once concrete arch types are available.

}

impl System {

    /// DESCRIPTION
    /// create a board-composed system instance.
    pub fn new() -> Self {
        Self { _reserved: () }
    }

    /// DESCRIPTION
    /// register a task with the system.
    pub fn spawn_task(
        &mut self,
        task_id: TaskId,
        stack_top: *mut u8,
        entry_point: extern "C" fn(*mut ()) -> !,
        entry_arg: *mut (),
    ) -> Result<()> {
        let _ = (task_id, stack_top, entry_point, entry_arg);

        // TODO: spawn task in kernel.

        Ok(())
    }

    /// DESCRIPTION
    /// request a cooperative yield.
    pub fn yield_now(&self) {
        loop {
            core::hint::spin_loop();
        }

        // TODO: forward to kernel yield path.

    }

    /// DESCRIPTION
    /// start the system runtime.
    pub fn run(&self) -> ! {
        loop {
            core::hint::spin_loop();
        }

        // TODO: "hand over control to scheduler/context-switch start pathk

    }
}