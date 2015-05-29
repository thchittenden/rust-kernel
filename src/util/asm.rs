// Would use bitflags! but that would just complicate things.
const CR0_PG: u32 = 1 << 31;
const CR4_PSE: u32 = 1 << 4;
const CR4_PGE: u32 = 1 << 7;


pub fn set_cr3(cr3: usize) {
    unsafe { asm!("mov $0, %cr3" :: "r"(cr3)) }
}

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
