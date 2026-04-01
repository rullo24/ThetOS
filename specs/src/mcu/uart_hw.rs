use core::result::Result;

/// low-level MCU UART capability contract (no board-facing behaviour).
pub trait UartHardware {
    type Error: core::fmt::Debug;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn read_byte(&mut self) -> Result<u8, Self::Error>;
}
