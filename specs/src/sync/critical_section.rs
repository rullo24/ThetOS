
/// executes a closure with interrupts masked, restoring the previous state on completion.
pub trait CriticalSection {
    fn withExecute<Res, Op>(&self, operation: Op) -> Res
    where Op: FnOnce() -> Res;
}