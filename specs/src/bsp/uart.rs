use core::result::Result;

/// board-facing serial contract exposed to application code.
pub trait UartPort {
    type Error: core::fmt::Debug;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn read_byte(&mut self) -> Result<u8, Self::Error>;
}
