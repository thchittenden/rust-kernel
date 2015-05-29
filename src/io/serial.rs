//!
//! This module contains definitions for interacting with the serial ports.
//!
use core::prelude::*;
use core::fmt::{Write, Arguments, Error};
use mutex::Mutex;
use util::asm;

const DATA_OFFSET: u16 = 0;
const INT_ENABLE_OFFSET: u16 = 1;
const DIV_LSB_OFFSET: u16 = 0;
const DIV_MSB_OFFSET: u16 = 1;
const FIFOCNT_OFFSET: u16 = 2;
const LCR_OFFSET: u16 = 3;
const MCR_OFFSET: u16 = 4;
const LSR_OFFSET: u16 = 5;
const MSR_OFFSET: u16 = 6;
const SCRATCH_OFFSET: u16 = 7;

const BAUD_TOP: u32 = 115200;

bitflags! {
    flags LCR: u8 {
        const DATA5       = 0b00000000,
        const DATA6       = 0b00000001,
        const DATA7       = 0b00000010,
        const DATA8       = 0b00000011,
        const STOP1       = 0b00000000,
        const STOP2       = 0b00000100,
        const PARITY_NONE = 0b00000000,
        const PARITY_ODD  = 0b00001000,
        const PARITY_EVEN = 0b00011000,
        const PARITY_HIGH = 0b00101000,
        const PARITY_LOW  = 0b00111000,
        const BREAK_EN    = 0b01000000,
        const DLAB_EN     = 0b10000000,
    }
}

// TODO: if CTFE is ever a thing, make this a bit prettier.
pub const LCR_8N1: LCR = LCR { bits: DATA8.bits | PARITY_NONE.bits | STOP1.bits };

bitflags! {
    flags LSR: u8 {
        const DATA_AVAILABLE = 0b00000001,
        const OVERRUN_ERROR  = 0b00000010,
        const PARITY_ERROR   = 0b00000100,
        const FRAMING_ERROR  = 0b00001000,
        const BREAK_RECVD    = 0b00010000,
        const THR_EMPTY      = 0b00100000,
        const THR_EMPTY_IDLE = 0b01000000,
        const BAD_FIFO       = 0b10000000,
    }
}

/// A serial port.
pub struct SerialPort {
    base: u16,
    baud: u32,
    lcr: LCR,
}

/// A thread-safe serial port.
pub struct SafeSerialPort {
    sp: Mutex<SerialPort>
}

impl SerialPort {

    /// Creates a new serial port at the given I/O address with the given baud and control flags.
    pub fn new(base: u16, baud: u32, lcr: LCR) -> SerialPort {
        let mut sp = SerialPort { base: base, baud: baud, lcr: lcr };
        sp.configure(baud, lcr); 
        sp
    }

    /// Configures the serial port at a new baud/control configuration.
    pub fn configure(&mut self, baud: u32, lcr: LCR) {
        self.set_lcr(lcr);
        self.set_baud(baud);
    }

    /// Writes a character to the serial port. This currently blocks until the transmit buffer is
    /// empty. It would be better to maintain an internal buffer that is interrupt-driven.
    pub fn putc(&self, c: char) {
        while !self.get_lsr().contains(THR_EMPTY) { 
            // Spin. TODO fix this when we have interrupts.
        }
        asm::outb8(self.base + DATA_OFFSET, c as u8);
    }

    /// Retrieves a character from the serial port. This currently blocks until the receive buffer
    /// is non-empty. This is completely unsafe and needs to be implemented using interrupts as it
    /// is far too easy to miss a character currently.
    pub fn getc(&self) -> char {
        while !self.get_lsr().contains(DATA_AVAILABLE) {
            // Spin. TODO fix this when we have interrupts.
            // This one is ESPECIALLY bad...
        }
        asm::inb8(self.base + DATA_OFFSET) as char
    }

    fn set_baud(&mut self, baud: u32) {
        assert!(baud != 0);
        assert!(BAUD_TOP % baud == 0);
        self.set_dlab(true);
        let div: u16 = (BAUD_TOP / baud) as u16; 
        asm::outb8(self.base + DIV_LSB_OFFSET, div as u8);
        asm::outb8(self.base + DIV_MSB_OFFSET, (div >> 8) as u8);
        self.set_dlab(false);
        self.baud = baud;
    }

    fn set_lcr(&mut self, lcr: LCR) {
        assert!(!lcr.contains(BREAK_EN));
        assert!(!lcr.contains(DLAB_EN));
        asm::outb8(self.base + LCR_OFFSET, lcr.bits);
        self.lcr = lcr;
    }
    
    fn set_dlab(&mut self, on: bool) {
        if on {
            self.lcr.insert(DLAB_EN);
        } else {
            self.lcr.remove(DLAB_EN);
        }
        asm::outb8(self.base + LCR_OFFSET, self.lcr.bits);
    }

    fn get_lsr(&self) -> LSR {
        LSR::from_bits(asm::inb8(self.base + LSR_OFFSET)).unwrap()
    }

}

impl Write for SerialPort {

    /// Writes a string to the serial port.
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.chars() {
            self.putc(c)
        }
        Ok(())
    }

}

impl SafeSerialPort {

    /// Constructs a new thread-safe serial port.
    pub fn new(base: u16, baud: u32, lcr: LCR) -> SafeSerialPort {
        SafeSerialPort { sp: static_mutex!(SerialPort::new(base, baud, lcr)) }
    }

    /// Atomically writes a string to the serial port.
    pub fn write_str(&self, s: &str) -> Result<(), Error> {
        let mut sp = self.sp.lock().unwrap();
        sp.write_str(s)
    }

    /// Atomically writes a format string to the serial port.
    pub fn write_fmt(&self, args: Arguments) -> Result<(), Error> {
        let mut sp = self.sp.lock().unwrap();
        sp.write_fmt(args)
    }

}
