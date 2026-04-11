use specs::arch::{StackGuard, StackGuardContext};

/// static stack pool, guard impl and per-task guard contexts
pub struct KernelStackResources<G: StackGuard + Copy> {
    pub stack_pool: &'static mut [u8],
    pub stack_guard: G,
    pub stack_guard_slots: &'static mut [Option<StackGuardContext>],
}

impl <G: StackGuard + Copy> KernelStackResources<G> {
    /// DESCRIPTION
    /// bunlde stack mem, guard impl and slot storage for `Kernel::new`
    pub fn new(
        stack_pool: &'static mut [u8],
        stack_guard: G,
        stack_guard_slots: &'static mut [Option<StackGuardContext>],
    ) -> Self {
        Self {
            stack_pool,
            stack_guard,
            stack_guard_slots,
        }
    } 
}

