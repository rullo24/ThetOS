// USART2 polled driver -> classic STM32L1 SR/DR USART (RM0038 rev 18 ch.27)
// PA2/PA3 (USART2 TX/RX) route to the ST-LINK virtual COM port on the Nucleo-L152RE
use core::marker::PhantomData;
use core::ptr::{read_volatile, write_volatile};

use crate::clock::PCLK1_HZ;
use crate::gpio::Uninit as PinUninit;
use crate::gpio::{Pin, PortA};

// board-facing UART contract + typestate markers, re-exported so a consumer never names `specs`
pub use specs::bsp::uart::{
    Active, Parity, StopBits, Uart, UartConfig, UartError, UartMode, Uninit, UninitUart,
};

// ---- USART2 register block (RM0038 rev 18 s27.6.8 Table; base from s2.3 Table 5) ----
const USART2_BASE: usize = 0x4000_4400;
const USART2_SR: *mut u32 = (USART2_BASE + 0x00) as *mut u32; // status | s27.6.1
const USART2_DR: *mut u32 = (USART2_BASE + 0x04) as *mut u32; // data | s27.6.2
const USART2_BRR: *mut u32 = (USART2_BASE + 0x08) as *mut u32; // baud rate | s27.6.3
const USART2_CR1: *mut u32 = (USART2_BASE + 0x0C) as *mut u32; // control 1 | s27.6.4
const USART2_CR2: *mut u32 = (USART2_BASE + 0x10) as *mut u32; // control 2 | s27.6.5

// ---- RCC APB1 peripheral clock enable (RM0038 rev 18 s6.3.10) ----
const RCC_APB1ENR: *mut u32 = 0x4002_3824 as *mut u32; // RCC base 0x4002_3800 + 0x24 offset
const RCC_APB1ENR_USART2EN: u32 = 1 << 17;

// ---- USART2_SR flags (RM0038 rev 18 s27.6.1) ----
const SR_PE: u32 = 1 << 0; // parity error
const SR_FE: u32 = 1 << 1; // framing error
const SR_ORE: u32 = 1 << 3; // overrun error
const SR_RXNE: u32 = 1 << 5; // read data register not empty
const SR_TC: u32 = 1 << 6; // transmission complete
const SR_TXE: u32 = 1 << 7; // transmit data register empty

// ---- USART2_CR1 bits (RM0038 rev 18 s27.6.4) ----
const CR1_RE: u32 = 1 << 2; // receiver enable
const CR1_TE: u32 = 1 << 3; // transmitter enable
const CR1_PS: u32 = 1 << 9; // parity selection: 0 even, 1 odd
const CR1_PCE: u32 = 1 << 10; // parity control enable
const CR1_M: u32 = 1 << 12; // word length: 1 -> 9 total bits, keeps 8 data bits when parity is on
const CR1_UE: u32 = 1 << 13; // USART enable

// ---- USART2_CR2 bits (RM0038 rev 18 s27.6.5) ----
const CR2_STOP_2BIT: u32 = 0b10 << 12; // STOP[13:12]: 00 -> 1 stop bit, 10 -> 2 stop bits

// USART2 alternate function for PA2/PA3 (STM32L152xE datasheet, alternate function table)
const USART2_AF: u8 = 7;

/// DESCRIPTION
/// USART2 handle -> MODE is Uninit until into_active() programs the hardware
pub struct Serial<MODE: UartMode> {
    _mode: PhantomData<MODE>,
}

impl Serial<Uninit> {
    /// DESCRIPTION
    /// take ownership of the two USART2 pins so they can't also be driven as GPIO, route them to AF7, hand back an unconfigured port
    pub fn usart2(tx: Pin<PortA, 2, PinUninit>, rx: Pin<PortA, 3, PinUninit>) -> Self {
        let _tx = tx.into_alternate(USART2_AF);
        let _rx = rx.into_alternate(USART2_AF);
        Serial { _mode: PhantomData }
    }
}

impl UninitUart for Serial<Uninit> {
    type Active = Serial<Active>;

    /// DESCRIPTION
    /// enable the peripheral clock, program baud / parity / stop bits, enable receiver and transmitter
    fn into_active(self, config: UartConfig) -> Serial<Active> {
        unsafe {
            write_volatile(
                RCC_APB1ENR,
                read_volatile(RCC_APB1ENR) | RCC_APB1ENR_USART2EN,
            );

            // BRR is USARTDIV in 12.4 fixed point (OVER8 = 0) -> its integer value is fCK / baud
            let brr = (PCLK1_HZ + config.baud / 2) / config.baud;
            write_volatile(USART2_BRR, brr);

            let cr2 = match config.stop {
                StopBits::One => 0,
                StopBits::Two => CR2_STOP_2BIT,
            };
            write_volatile(USART2_CR2, cr2);

            let mut cr1 = CR1_UE | CR1_TE | CR1_RE;
            match config.parity {
                Parity::None => {}
                // the parity bit occupies the frame MSB, so widen the word to keep 8 data bits
                Parity::Even => cr1 |= CR1_PCE | CR1_M,
                Parity::Odd => cr1 |= CR1_PCE | CR1_PS | CR1_M,
            }
            write_volatile(USART2_CR1, cr1);
        }
        Serial { _mode: PhantomData }
    }
}

impl Uart for Serial<Active> {
    /// DESCRIPTION
    /// spin until the data register can accept a byte, then write it
    fn write_byte(&mut self, byte: u8) {
        unsafe {
            while read_volatile(USART2_SR) & SR_TXE == 0 {}
            write_volatile(USART2_DR, byte as u32);
        }
    }

    /// DESCRIPTION
    /// spin until the final byte has fully shifted out of the transmitter
    fn flush(&mut self) {
        unsafe { while read_volatile(USART2_SR) & SR_TC == 0 {} }
    }

    /// DESCRIPTION
    /// check receive faults first (they latch with the offending byte), then RXNE
    fn read_byte(&mut self) -> Result<Option<u8>, UartError> {
        unsafe {
            let sr = read_volatile(USART2_SR);
            if sr & SR_ORE != 0 {
                let _ = read_volatile(USART2_DR); // SR read then DR read clears ORE
                return Err(UartError::Overrun);
            }
            if sr & SR_FE != 0 {
                let _ = read_volatile(USART2_DR);
                return Err(UartError::Framing);
            }
            if sr & SR_PE != 0 {
                let _ = read_volatile(USART2_DR);
                return Err(UartError::Parity);
            }
            if sr & SR_RXNE == 0 {
                return Ok(None);
            }
            Ok(Some(read_volatile(USART2_DR) as u8))
        }
    }
}
