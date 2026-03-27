#![no_std]

use specs::cpu::ContextSwitch;
use specs::sync::CriticalSection;
use specs::scheduler::TaskId;
use specs::error::{ThetosError, Result};

// hardware-blind kernel
pub struct Kernel<CtxSwitchType, CriticalSectionType> 
where
    CtxSwitchType: ContextSwitch,
    CriticalSectionType: CriticalSection,
{
    ctx_switch: CtxSwitchType,
    crit_section: CriticalSectionType,
    curr_task: Option<TaskId>,
    task_count: usize,
}

impl<CtxSwitchType, CriticalSectionType> Kernel<CtxSwitchType, CriticalSectionType>
where
    CtxSwitchType: ContextSwitch,
    CriticalSectionType: CriticalSection,
{
    /// DESCRIPTION
    /// create a new hardware-blind kernel instance
    pub fn new(ctx_switch: CtxSwitchType, crit_section: CriticalSectionType) -> Self {
        return Self {
            ctx_switch,
            crit_section,
            curr_task: None,
            task_count: 0,
        };
    }

    /// DESCRIPTION
    /// spawn a new task w/ registered context
    pub fn spawn_task(
        &mut self,
        task_id: TaskId,
        stack_top: *mut u8,
        entry_point: extern "C" fn(*mut ()),
        entry_arg: *mut (),
    ) -> Result<()> {

        // check if stack top is valid
        if stack_top.is_null() {
            return Err(ThetosError::InvalidConfig);
        }

        // initialise task context for the new task
        let _ = self.ctx_switch.initialiseTaskContext(stack_top, entry_point, entry_arg);
        self.task_count += 1; 

        // checking if no task is currently running
        if self.curr_task.is_none() {
            self.curr_task = Some(task_id); // setting the new task as the current task
        }

        return Ok(()); // success
    }


    /// DESCRIPTION
    /// trigger a voluntary context switch
    pub fn yield_now(&self) {
        self.ctx_switch.triggerPendSwitch();
    }

    /// DESCRIPTION
    /// get the currently processes task
    pub fn get_current_task(&self) -> Option<TaskId> {
        return self.curr_task;
    }

    /// DESCRIPTION
    /// return the num of tasks registered
    pub fn get_task_count(&self) -> usize {
        return self.task_count;
    }

    /// DESCRIPTION
    /// execute an operation inside a critical section.
    pub fn execute_in_critical_section<Res, Op>(&self, operation: Op) -> Res 
    where Op: FnOnce() -> Res, // called at least once before return
    {
        return self.crit_section.withExecute(operation);
    }

}