
/// disable/enable all IRQs
pub trait InterruptControl {
    fn mask_irqs(&mut self);
    fn unmask_irqs(&mut self);
}
