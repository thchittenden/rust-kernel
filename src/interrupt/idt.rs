use core::prelude::*;
use util::{asm, KERNEL_CODE_SEGMENT};
use util::global::Global;
use util::rawbox::RawBox;

bitflags! {
    flags GateFlags: u16 {
        const EMPTY       = 0b00000000_00000000,
        const PRESENT     = 0b10000000_00000000,
        const INTERRUPT32 = 0b00001110_00000000,
        const INTERRUPT16 = 0b00000110_00000000,
        const TRAP32      = 0b00001111_00000000,
        const TRAP16      = 0b00000111_00000000,
        const DPL0        = 0b00000000_00000000,
        const DPL1        = 0b00100000_00000000,
        const DPL2        = 0b01000000_00000000,
        const DPL3        = 0b01100000_00000000,
    }
}

#[repr(C, packed)]
struct IDTEntry {
    offset_lo: u16,
    segment: u16,
    flags: GateFlags, 
    offset_hi: u16
}

impl IDTEntry {

    pub fn init(&mut self, wrapper: usize, flags: GateFlags) {
        // Clear flags so that we can't get an interrupt mid modification.
        self.flags     = EMPTY;

        // Set up the descriptor.
        self.offset_lo = wrapper as u16;
        self.offset_hi = (wrapper >> 16) as u16;
        self.segment   = KERNEL_CODE_SEGMENT;
        self.flags     = flags | PRESENT;
    }

    pub fn is_present(&self) -> bool {
        self.flags.contains(PRESENT)
    }

}

struct IDT {
    entries: [IDTEntry; 256]
}

static mut IDT: Global<RawBox<IDT>> = global_init!();

/// Initializes an IDT entry. It would be great if we could perform a concat_idents on the name
/// with _isr_wrapper so we don't have to write it every time but MACROS DON'T WORK ON EXTERN
/// ITEMS.
macro_rules! idt_entry {
    ($id:expr, $name:ident, $flags:expr) => ({
        let wrapper = linker_sym!($name);
        IDT.entries[$id].init(wrapper, $flags);
    });
}

pub fn init_idt() {
    // We know this is safe because only the interrupt module touches the IDT.
    unsafe { 
        let idt_ptr = RawBox::from_raw(linker_sym!(_idt) as *mut IDT);
        IDT.init(idt_ptr);
        
        // x86 Core Interrupts.
        idt_entry!(0,  _isr_wrapper_DIVIDE_ERROR,       TRAP32 | DPL0);

        idt_entry!(2,  _isr_wrapper_NMI,                TRAP32 | DPL0);
        idt_entry!(3,  _isr_wrapper_BREAKPOINT,         TRAP32 | DPL0);
        idt_entry!(4,  _isr_wrapper_OVERFLOW,           TRAP32 | DPL0);
        idt_entry!(5,  _isr_wrapper_BOUND,              TRAP32 | DPL0);
        idt_entry!(6,  _isr_wrapper_INV_OPCODE,         TRAP32 | DPL0);
        idt_entry!(7,  _isr_wrapper_NO_MATH,            TRAP32 | DPL0);

        idt_entry!(9,  _isr_wrapper_COPROC_OVERRUN,     TRAP32 | DPL0);
        idt_entry!(10, _isr_wrapper_INVALID_TSS,        TRAP32 | DPL0);
        idt_entry!(11, _isr_wrapper_NOT_PRESENT,        TRAP32 | DPL0);
        idt_entry!(12, _isr_wrapper_STACK_SEG_FAULT,    TRAP32 | DPL0);
        idt_entry!(13, _isr_wrapper_PROTECTION_FAULT,   TRAP32 | DPL0);
        idt_entry!(14, _isr_wrapper_PAGE_FAULT,         INTERRUPT32 | DPL0);

        idt_entry!(16, _isr_wrapper_MATH_FAULT,         TRAP32 | DPL0);
        idt_entry!(17, _isr_wrapper_ALIGNMENT_FAULT,    TRAP32 | DPL0);
        idt_entry!(18, _isr_wrapper_MACHINE_CHECK,      TRAP32 | DPL0);
        idt_entry!(19, _isr_wrapper_SIMD_FAULT,         TRAP32 | DPL0);
        idt_entry!(20, _isr_wrapper_VIRT_FAULT,         TRAP32 | DPL0);
    
        // 8259-PIC interrupts.
        idt_entry!(32, _isr_wrapper_TIMER_INT,          INTERRUPT32 | DPL0);
        idt_entry!(33, _isr_wrapper_KEYBOARD_INT,       INTERRUPT32 | DPL0);

        idt_entry!(35, _isr_wrapper_SERIAL24_INT,       INTERRUPT32 | DPL0);
        idt_entry!(36, _isr_wrapper_SERIAL13_INT,       INTERRUPT32 | DPL0);
        idt_entry!(37, _isr_wrapper_PARALLEL2_INT,      INTERRUPT32 | DPL0);
        idt_entry!(38, _isr_wrapper_FLOPPY_INT,         INTERRUPT32 | DPL0);
        idt_entry!(39, _isr_wrapper_PARALLEL1_INT,      INTERRUPT32 | DPL0);
        idt_entry!(40, _isr_wrapper_RTC_INT,            INTERRUPT32 | DPL0);
        idt_entry!(41, _isr_wrapper_ACPI_INT,           INTERRUPT32 | DPL0);
        idt_entry!(42, _isr_wrapper_UNUSED1_INT,        INTERRUPT32 | DPL0);
        idt_entry!(43, _isr_wrapper_UNUSED2_INT,        INTERRUPT32 | DPL0);
        idt_entry!(44, _isr_wrapper_PS2_INT,            INTERRUPT32 | DPL0);
        idt_entry!(45, _isr_wrapper_FPU_INT,            INTERRUPT32 | DPL0);
        idt_entry!(46, _isr_wrapper_PRIMARY_ATA_INT,    INTERRUPT32 | DPL0);
        idt_entry!(47, _isr_wrapper_SECONDARY_ATA_INT,  INTERRUPT32 | DPL0);
    }
}


