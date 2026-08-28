use core::marker::PhantomData;
use core::ptr::{read_volatile, write_volatile};

use specs::bsp::gpio::{
    GpioLevel, Input, InputPin, Output, OutputPin, OutputStyle, PinMode, PullState, Uninit, UninitPin,
};

use super::port::GpioPort;

// GPIO register field encodings | Ref: RM0038 7.4
const MODER_INPUT: u32 = 0b00;
const MODER_OUTPUT: u32 = 0b01;
const PUPDR_NONE: u32 = 0b00;
const PUPDR_UP: u32 = 0b01;
const PUPDR_DOWN: u32 = 0b10;
const TWO_BIT_MASK: u32 = 0b11;

pub struct Pin<PORT: GpioPort, const PIN_INDEX: u8, MODE: PinMode> {
    _marker: PhantomData<(PORT, MODE)>,
}

impl<PORT: GpioPort, const PIN_INDEX: u8, MODE: PinMode> Pin<PORT, PIN_INDEX, MODE> {
    /// DESCRIPTION
    /// enable this port's CLK, then read it back
    fn enable_clk() {
        let rcc = PORT::RCC_ENABLE;
        unsafe {
            write_volatile(rcc.reg, read_volatile(rcc.reg) | (1u32 << rcc.bit));
            let _ = read_volatile(rcc.reg);
        }
    }
}

impl<PORT: GpioPort, const PIN_INDEX: u8> Pin<PORT, PIN_INDEX, Uninit> {
    /// DESCRIPTION
    /// pre-config handle -> doesn't touch hardware
    pub const fn new() -> Self {
        assert!(PIN_INDEX < 16, "GPIO pin index must be sub-16");
        Self {
            _marker: PhantomData,
        }
    }
}

impl<PORT: GpioPort, const PIN_INDEX: u8> Default for Pin<PORT, PIN_INDEX, Uninit> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PORT: GpioPort, const PIN_INDEX: u8> UninitPin for Pin<PORT, PIN_INDEX, Uninit> {
    type Input = Pin<PORT, PIN_INDEX, Input>;
    type Output = Pin<PORT, PIN_INDEX, Output>;

    /// DESCRIPTION
    /// configures the pin in the INPUT dir
    fn into_input(self, pull: PullState) -> Self::Input {
        Self::enable_clk();
        let field = (PIN_INDEX as u32) * 2; // 2 bits per pin field
        let pupd = match pull {
            PullState::HighZ => PUPDR_NONE,
            PullState::PullDown => PUPDR_DOWN,
            PullState::PullUp => PUPDR_UP,
        };
        unsafe {
            let curr_moder = read_volatile(PORT::MODER) & !(TWO_BIT_MASK << field);
            write_volatile(PORT::MODER, curr_moder | (MODER_INPUT << field));
            let curr_pupdr = read_volatile(PORT::PUPDR) & !(TWO_BIT_MASK << field);
            write_volatile(PORT::PUPDR, curr_pupdr | (pupd << field));
        }
        Pin {
            _marker: PhantomData,
        }
    }

    /// DESCRIPTION
    /// configures the pin in the OUTPUT dir
    fn into_output(self, style: OutputStyle) -> Self::Output {
        Self::enable_clk();
        let field = (PIN_INDEX as u32) * 2;
        let bit = PIN_INDEX as u32;
        unsafe {
            write_volatile(PORT::BSRR, 1u32 << (bit + 16)); // BSRR reset half -> start LOW
            let curr_otyper = read_volatile(PORT::OTYPER);
            let curr_otyper = match style {
                OutputStyle::PushPull => curr_otyper & !(1u32 << bit),
                OutputStyle::OpenDrain => curr_otyper | (1u32 << bit),
            };
            write_volatile(PORT::OTYPER, curr_otyper);
            let curr_moder = read_volatile(PORT::MODER) & !(TWO_BIT_MASK << field);
            write_volatile(PORT::MODER, curr_moder | (MODER_OUTPUT << field));
        }
        Pin {
            _marker: PhantomData,
        }
    }
}

impl<PORT: GpioPort, const PIN_INDEX: u8> OutputPin for Pin<PORT, PIN_INDEX, Output> {
    /// DESCRIPTION
    /// Sets the logical level of the pin to a GpioLevel
    fn set(&mut self, level: GpioLevel) {
        let bit = PIN_INDEX as u32;
        let write = match level {
            GpioLevel::High => 1u32 << bit,       // BSRR low half sets
            GpioLevel::Low => 1u32 << (bit + 16), // BSRR high half resets
        };
        unsafe { write_volatile(PORT::BSRR, write) };
    }
}

impl<PORT: GpioPort, const PIN_INDEX: u8> InputPin for Pin<PORT, PIN_INDEX, Input> {
    /// DESCRIPTION
    /// Reads the current logical level of the pin and returns it
    fn read(&self) -> GpioLevel {
        let high = unsafe { (read_volatile(PORT::IDR) >> (PIN_INDEX as u32)) & 1 == 1 };
        if high {
            GpioLevel::High
        } else {
            GpioLevel::Low
        }
    }
}
