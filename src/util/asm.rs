// Would use bitflags! but that would just complicate things.
const CR0_PG: u32 = 1 << 31;
const CR4_PSE: u32 = 1 << 4;
const CR4_PGE: u32 = 1 << 7;

const IF_FLAG: u32 = 1 << 9;

/// Returns the current EFLAGS register.
pub fn get_eflags() -> u32 {
    let mut eflags: u32;
    unsafe { asm! ("pushf\n\t
                    pop %eax\n\t
                    mov %eax, $0\n\t"
                   : "=r"(eflags)
                   :
                   : "eax")
    }
    eflags
}

/// Returns whether interrupts are enabled or not.
pub fn interrupts_enabled() -> bool {
    get_eflags() & IF_FLAG != 0
}

/// Enables interrupts.
pub fn enable_interrupts() {
    unsafe { asm!("sti") }
}

/// Disables interrupts.
pub fn disable_interrupts() {
    unsafe { asm!("cli") }
}

/// Sets the CR3 register to the given value.
pub fn set_cr3(cr3: usize) {
    unsafe { asm!("mov $0, %cr3" :: "r"(cr3)) }
}

pub fn get_cr3() -> usize {
    let mut cr3;
    unsafe { asm!("mov %cr3, $0" : "=r"(cr3)) };
    cr3
}

/// Enables paging.
pub fn enable_paging() {
    unsafe { 
        asm!("mov %cr0, %eax\n\t
              or $0, %eax\n\t
              mov %eax, %cr0\n\t" 
             :
             : "r"(CR0_PG)
             : "eax") 
    }
}

/// Enables page directories to contain 4MB regions.
pub fn enable_4mb_pages() {
    unsafe {
        asm!("mov %cr4, %eax\n\t
              or $0, %eax\n\t
              mov %eax, %cr4\n\t"
             : 
             : "r"(CR4_PSE)
             : "eax")
    }
}

/// Enable page table global bit.
pub fn enable_global_pages() {
    unsafe {
        asm!("mov %cr4, %eax\n\t
              or $0, %eax\n\t
              mov %eax, %cr4\n\t"
             : 
             : "r"(CR4_PGE)
             : "eax")
    }
}

/// Write 4 bytes to an I/O address.
pub fn outb32(addr: u16, val: u32) {
    unsafe {
        asm!("mov $0, %dx\n\t
              mov $1, %eax\n\t
              outl %eax, %dx\n\t"
            : 
            : "r"(addr), "r"(val)
            : "eax", "edx")
    }
}

/// Read 4 bytes from an I/O address.
pub fn inb32(addr: u16) -> u32 {
    let mut res: u32;
    unsafe {
        asm!("mov $1, %dx\n\t
              inl %dx, %eax\n\t
              mov %eax, $0\n\t"
            : "=r"(res)
            : "r"(addr)
            : "eax", "edx")
    }
    res

}

/// Write 8 bytes to an I/O address.
pub fn outb8(addr: u16, val: u8) {
    unsafe { 
        asm!("mov $0, %dx\n\t
              mov $1, %al\n\t
              outb %al, %dx\n\t"
            : 
            : "r"(addr), "r"(val)
            : "eax", "edx")
    }
}

/// Read 8 bytes from an I/O address.
pub fn inb8(addr: u16) -> u8 {
    let mut res: u8;
    unsafe {
        asm!("mov $1, %dx\n\t
              inb %dx, %al\n\t
              mov %al, $0\n\t"
            : "=r"(res)
            : "r"(addr)
            : "eax", "edx")
    }
    res
}
