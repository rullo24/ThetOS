
/// executes a closure with interrupts masked, restoring the previous state on completion.
pub trait CriticalSection {
    fn with_execute<Res, Op>(&self, operation: Op) -> Res
    where Op: FnOnce() -> Res;
}
