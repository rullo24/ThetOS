use core::marker::PhantomData;
use core::ptr::{read_volatile, write_volatile};

use specs::bsp::gpio::{
    GpioLevel, Input, InputPin, Output, OutputPin, OutputStyle, PinMode, PullState, Uninit,
    UninitPin,
};

use super::port::GpioPort;

// GPIO register field encodings, identical for every port | Ref: RM0038 rev 18 ch.7
const MODER_INPUT: u32 = 0b00; // GPIOx_MODER field: input mode | s7.4.1
const MODER_OUTPUT: u32 = 0b01; // GPIOx_MODER field: general purpose output mode | s7.4.1
const PUPDR_NONE: u32 = 0b00; // GPIOx_PUPDR field: no pull-up / pull-down | s7.4.4
const PUPDR_UP: u32 = 0b01; // GPIOx_PUPDR field: pull-up | s7.4.4
const PUPDR_DOWN: u32 = 0b10; // GPIOx_PUPDR field: pull-down | s7.4.4
const TWO_BIT_MASK: u32 = 0b11; // width of one MODER / PUPDR field (2 bits per pin)
                                // GPIOx_OTYPER: 0 = push-pull, 1 = open-drain, 1 bit per pin | s7.4.2
                                // GPIOx_OSPEEDR: left at reset (00 = low speed), adequate for on/off signalling | s7.4.3
                                // GPIOx_BSRR: low half sets ODR, high half (bit + 16) resets ODR, write-only | s7.4.7
                                // GPIOx_IDR: one read-only bit per pin | s7.4.5

pub struct Pin<PORT: GpioPort, const PIN_INDEX: u8, MODE: PinMode> {
    _marker: PhantomData<(PORT, MODE)>,
}

// helpers shared by a pin in any mode
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

// pre-configuration state -> build a handle without touching hardware
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

// Default forwards to new() -> lets an Uninit pin sit in a derived struct
impl<PORT: GpioPort, const PIN_INDEX: u8> Default for Pin<PORT, PIN_INDEX, Uninit> {
    /// DESCRIPTION
    /// same as new() -> an unconfigured pin handle
    fn default() -> Self {
        Self::new()
    }
}

// Uninit -> configured: the only impl that writes MODER / OTYPER / PUPDR
// each write is read -> clear this pin's field -> OR in the new value, so other pins'
// config is preserved (port A resets PA13/14/15 to SWD alternate-function + pulls)
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

// output-mode behaviour -> level writes go through BSRR
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

// input-mode behaviour -> level reads come from IDR
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
