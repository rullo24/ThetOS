
use core::result::Result;

pub trait Uart {
    type Error: core::fmt::Debug;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn read_byte(&mut self) -> Result<u8, Self::Error>;
}
