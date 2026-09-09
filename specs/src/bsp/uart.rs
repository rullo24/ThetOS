mod sealed {
    pub trait Sealed {}
}

// typestate marker
pub trait UartMode: sealed::Sealed {}

pub struct Uninit; // not configured yet
pub struct Active;

impl sealed::Sealed for Uninit {}
impl sealed::Sealed for Active {}
impl UartMode for Uninit {}
impl UartMode for Active {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Parity {
    None,
    Odd,
    Even,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StopBits {
    One,
    Two,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UartError {
    Overrun, // a byte arrived before previous one was read
    Framing, // stop bit not seen
    Parity,  // parity check failed
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UartConfig {
    pub baud: u32,
    pub parity: Parity,
    pub stop: StopBits,
}

impl Default for UartConfig {
    /// DESCRIPTION
    /// 115200 8N1
    fn default() -> Self {
        Self {
            baud: 115_200,
            parity: Parity::None,
            stop: StopBits::One,
        }
    }
}

pub trait UninitUart {
    type Active: Uart;

    /// DESCRIPTION
    /// Configure the peripheral and its pins, returning the active port
    fn into_active(self, config: UartConfig) -> Self::Active;
}

pub trait Uart {
    /// DESCRIPTION
    /// Blocks until the byte is accepted by the transmitter
    fn write_byte(&mut self, byte: u8);

    /// DESCRIPTION
    /// Blocks writing every byte in order
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    /// DESCRIPTION
    /// Blocks until the transmit shift register is empty
    fn flush(&mut self);

    /// DESCRIPTION
    /// Ok(None) if no byte is waiting; Err on a receive fault
    fn read_byte(&mut self) -> Result<Option<u8>, UartError>;
}
