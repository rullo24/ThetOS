
/// disable/enable all IRQs
pub trait InterruptControl {
    fn maskIrqs(&mut self);
    fn unmaskIrqs(&mut self);
}
