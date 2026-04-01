use core::result::Result;

/// board-facing UART resource contract exposed to application code.
pub trait BoardUart {
    type Error: core::fmt::Debug;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn read_byte(&mut self) -> Result<u8, Self::Error>;
}
