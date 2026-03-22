
/// disable/enable all IRQs
pub trait InterruptControl {
    fn maskIrqs(&mut self);
    fn unmaskIrqs(&mut self);

    // TODO: add more methods as required by hardware
}