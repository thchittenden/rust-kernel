#![crate_name="interrupt"]
#![crate_type="rlib"]
#![feature(no_std,core,concat_idents)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate mutex;
extern crate alloc;
extern crate mem;

mod pic;
mod idt;
mod timer;

use core::prelude::*;
use core::mem::size_of;
use mutex::Mutex;
use idt::init_idt;
use pic::init_pic;
use timer::init_timer;
use util::asm;
use alloc::boxed::Box;

pub struct Regs {
    pub edi: u32, 
    pub esi: u32,
    pub ebp: u32,
    esp: u32, // This is not the REAL esp.
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
}

pub struct IRet {
    pub error_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub esp: u32,
    pub ss: u32,
}

pub type ISR = &'static Fn(u8, &mut Regs, &mut IRet);
struct IVT {
    vectors: [Option<ISR>; 256],
}
unsafe impl Sync for IVT { }
static mut IVT: IVT = IVT { vectors: [None; 256] };

pub fn init() {
    init_pic();
    init_idt();
    init_timer();
}

pub fn set_isr(irq: u8, isr: ISR) {
    // We know this is safe because the only place we only assign to this table with interrupts
    // disabled.
    assert!(!asm::interrupts_enabled());
    unsafe { IVT.vectors[irq as usize] = Some(isr) }; 
}

/// The interrupt dispatcher. This is called by all interrupt wrappers and dispatches the interrupt
/// to the appropriate interrupt vector.
#[no_mangle]
pub extern fn rust_interrupt_dispatch (irq: u8, regs: &mut Regs, ret: &mut IRet) {
    // We know this is safe because we only assign to this table with interrupts disabled.
    match unsafe { IVT.vectors[irq as usize] } {
        Some(isr) => isr(irq, regs, ret),
        None      => panic!("unhandled interrupt {}", irq)
    };
}
