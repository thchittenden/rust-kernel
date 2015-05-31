mod keyhelp;

use core::prelude::*;
use interrupt::{pic, Regs, IRet};
use util::asm;
logger_init!(Trace);

const KEYBOARD_PORT: u16 = 0x60;

pub fn keyboard_handler(id: u8, _: &mut Regs, _: &mut IRet) {
    let key = asm::inb8(KEYBOARD_PORT);
    let c = keyhelp::process_key(key as isize);
    pic::acknowledge_irq(id);
}

pub fn getc() -> char {
   unimplemented!() 
}
