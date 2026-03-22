
/// called when the scheduler has nothing ready (implementation is arch/ specific)
pub trait Idle {
    fn waitForInterrupt(&mut self);

    // TODO: add more methods as required by hardware
}