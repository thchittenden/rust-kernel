mod keyhelp;
mod buf;

use core::prelude::*;
use interrupt::{pic, Regs, IRet};
use util::asm;
use self::buf::{KeyboardBuffer, KEYBOARD_BUFFER_INIT};

const KEYBOARD_PORT: u16 = 0x60;
static mut KEYBOARD_BUF: KeyboardBuffer = KEYBOARD_BUFFER_INIT;

/// Handles a keyboard interrupt. Enqueues a character into the keyboard 
/// buffer if this interrupt generated one.
pub fn keyboard_handler(id: u8, _: &mut Regs, _: &mut IRet) {
    let key = asm::inb8(KEYBOARD_PORT);
    let res = keyhelp::process_key(key);

    // We know this is safe because only the interrupt handler ever enqueues.
    if let Some(c) = res {
        unsafe { KEYBOARD_BUF.enqueue(c) };
    }

    pic::acknowledge_irq(id);
}

/// Gets a character from the keyboard. Blocks until a character is available.
pub fn getc() -> char {
    loop {
        if let Some(c) = unsafe { KEYBOARD_BUF.dequeue() } {
            return c;
        }
    }
}
