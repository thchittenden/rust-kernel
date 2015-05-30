use util::asm;

const TIMER_CHAN0: u16 = 0x0040;
const TIMER_CHAN1: u16 = 0x0041;
const TIMER_CHAN2: u16 = 0x0042;
const TIMER_COMM: u16 = 0x0043;

/// The timer frequency in hertz.
const TIMER_FREQ: u32 = 1_193_182;

/// The desired interrupt frequency in hertz.
const INT_FREQ: u32 = 1_000;

/// The timer divider.
const TIMER_DIV: u32 = TIMER_FREQ / INT_FREQ;

bitflags! {
    flags TimerCommand: u8 {
        const Binary = 0b0000_0000,
        const BCD    = 0b0000_0001,
        const Mode0  = 0b0000_0000, // Interrupt on terminal count.
        const Mode1  = 0b0000_0010, // Hardware one shot.
        const Mode2  = 0b0000_0100, // Rate generator.
        const Mode3  = 0b0000_0110, // Square wave.
        const Mode4  = 0b0000_1000, // Software strobe.
        const Mode5  = 0b0000_1010, // Hardware strobe.
        const LoOnly = 0b0001_0000,
        const HiOnly = 0b0010_0000,
        const LoHi   = 0b0011_0000,
        const Chan0  = 0b0000_0000,
        const Chan1  = 0b0100_0000,
        const Chan2  = 0b1000_0000,
    }
}

pub fn init_timer() {
    set_frequency(INT_FREQ);
}

/// Sets the frequency of the timer.
///
/// # Panics
///
/// Panics if the requested frequency cannot be set.
pub fn set_frequency(req_freq: u32) {
    let div = TIMER_FREQ / req_freq;
    assert!(div <= u16::max_value() as u32);
    let div_lo = getbyte!(div, 0);
    let div_hi = getbyte!(div, 1);
    let command = (Binary | Mode3 | LoHi | Chan0).bits;
    asm::outb8(TIMER_COMM, command);
    asm::outb8(TIMER_CHAN0, div_lo);
    asm::outb8(TIMER_CHAN0, div_hi);
}
