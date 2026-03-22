
use core::result::Result;

pub trait Uart {
    type Error: core::fmt::Debug;

    fn writeByte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn readByte(&mut self) -> Result<u8, Self::Error>;

    // TODO: add more methods as required by hardware
}