//! This module defines the x86 interrupt interface. 
//! 
//! Interrupt handler wrappers are found in wrapper.S. These wrappers all call the Rust interrupt
//! dispatch routine which calls the relevent handler contained in the Interrupt Vector Table. By
//! doing this, there is one common entry point for all interrupts in Rust code.
//!
//! The wrappers push all GP registers and pass pointers to the PUSHA and IRET stack regions which
//! gives all interrupt handlers full control of where they return to.
//!
#![crate_name="interrupt"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,const_fn)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;

/// The 8259PIC driver.
pub mod pic;

/// The clock driver.
pub mod timer;

mod idt;

use core::prelude::*;
use timer::init_timer;
use pic::init_pic;
use idt::init_idt;
use util::{asm, KERNEL_CODE_SEGMENT};
logger_init!(Trace);

// x86 Core Interrupts.
pub const DIVIDE_ERROR_IRQ: u8          = 0;
pub const NMI_IRQ: u8                   = 2;
pub const BREAKPOINT_IRQ: u8            = 3;
pub const OVERFLOW_IRQ: u8              = 4;
pub const BOUND_IRQ: u8                 = 5;
pub const INV_OPCODE_IRQ: u8            = 6;
pub const NO_MATH_IRQ: u8               = 7;
pub const COPROC_OVERRUN_IRQ: u8        = 9;
pub const INVALID_TSS_IRQ: u8           = 10;
pub const NOT_PRESENT_IRQ: u8           = 11;
pub const STACK_SEG_FAULT_IRQ: u8       = 12;
pub const PROTECTION_FAULT_IRQ: u8      = 13;
pub const PAGE_FAULT_IRQ: u8            = 14;
pub const MATH_FAULT_IRQ: u8            = 16;
pub const ALIGNMENT_FAULT_IRQ: u8       = 17;
pub const MACHINE_CHECK_IRQ: u8         = 18;
pub const SIMD_FAULT_IRQ: u8            = 19;
pub const VIRT_FAULT_IRQ: u8            = 20;

// PIC Interrupts.
pub const TIMER_INT_IRQ: u8             = 32;
pub const KEYBOARD_INT_IRQ: u8          = 33;
pub const SERIAL24_INT_IRQ: u8          = 35;
pub const SERIAL13_INT_IRQ: u8          = 36;
pub const PARALLEL2_INT_IRQ: u8         = 37;
pub const FLOPPY_INT_IRQ: u8            = 38;
pub const PARALLEL1_INT_IRQ: u8         = 39;
pub const RTC_INT_IRQ: u8               = 40;
pub const ACPI_INT_IRQ: u8              = 41;
pub const UNUSED1_INT_IRQ: u8           = 42;
pub const UNUSED2_INT_IRQ: u8           = 43;
pub const PS2_INT_IRQ: u8               = 44;
pub const FPU_INT_IRQ: u8               = 45;
pub const PRIMARY_ATA_INT_IRQ: u8       = 46;
pub const SECONDARY_ATA_INT_IRQ: u8     = 47;

/// The GP registers pushed during a `pusha` instruction. It is important to not let the user
/// access the `esp` field because it does not correspond to any useful information, only where the
/// stack pointer was before the `pusha` instruction. The actual `esp` is in the IRet struct.
#[repr(C, packed)]
#[allow(missing_docs)]
pub struct Regs {
    pub edi: u32, 
    pub esi: u32,
    pub ebp: u32,
    #[allow(dead_code)] esp: u32, // This is not the REAL esp.
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
}

/// Encapuslates the data pushed to stack during an interrupt. It is important to prevent users
/// from manually accessing the ESP/SS entries since they will not be valid if the interrupt
/// occurred in kernel land.
#[repr(C, packed)]
pub struct IRet {
    pub error_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    esp: u32,       // ONLY VALID IF CS == USER_CODE_SEGMENT
    ss: u32,        // ONLY VALID IF CS == USER_CODE_SEGMENT
}

impl IRet {
  
    /// Returns whether the interrupt came from kernel mode or not.
    pub fn in_kernel(&self) -> bool {
        self.cs as u16 == KERNEL_CODE_SEGMENT
    }

    /// Tries to return the user land stack pointer in the IRet struct. If the interrupt occurred
    /// while in kernel land then ESP is not pushed to the stack.
    pub fn get_esp(&self) -> Option<u32> {
        if self.in_kernel() {
            None
        } else {
            Some(self.esp)
        }
    }

    /// Tries to return the user land stack segment in the IRet struct. If the interrupt occurred
    /// while in kernel land then SS is not pushed to the stack.
    pub fn get_ss(&self) -> Option<u32> {
        if self.in_kernel() {
            None
        } else {
            Some(self.esp)
        }
    }

}   

/// An Interrupt Service Routine. These functions take the interrupt number and mutable references
/// to the registers and return information. This allows them to fully control where they return
/// to.
pub type ISR = fn(u8, &mut Regs, &mut IRet);

/// An interrupt vector table. This contains all registered ISRs. We manually mark it as Sync so we
/// can access it mutably and globally.
struct IVT {
    vectors: [Option<ISR>; 256],
}
unsafe impl Sync for IVT { }
static mut IVT: IVT = IVT { vectors: [None; 256] };

/// Sets an interrupt number's handler to the given ISR.
pub fn set_isr(irq: u8, isr: ISR) {
    // We know this is safe because the only place we only assign to this table with interrupts
    // disabled.
    assert!(!asm::interrupts_enabled());
    unsafe { 
        assert!(IVT.vectors[irq as usize].is_none());
        IVT.vectors[irq as usize] = Some(isr);
    }
}

/// Initializes the interrupt module. 
pub fn init() {
    debug!("initializing interrupt");
    init_pic();
    init_idt();
    init_timer();
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
