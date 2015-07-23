mod constants;

use alloc::boxed::Box;
use collections::vec::Vec;
use core::prelude::*;
use core::mem;
use fs::File;
use mem::virt::pt::{PTE_WRITABLE, PTE_NONE};
use mem::virt::AddressSpace;
use self::constants::*;
use util::{page_align, page_align_up, PAGE_SIZE, KernResult};
use util::KernError::*;
use ::Loadable;

#[derive(Default)]
#[repr(C, packed)]
struct ELF32Header {
    ident_magic: u32,       // Initial magic. Always 0x464c457f.
    ident_class: u8,        // The ELF class. 1 for 32-bit, 2 for 64-bit.
    ident_data: u8,         // The data encoding. 1 for LSB, 2 for MSB.
    ident_version: u8,      // The ELF header version. Always 1.
    ident_padding0: u8,     // Padding. Possibly OSABI.
    ident_padding1: u32,
    ident_padding2: u32,

    e_type: u16,            // The object file type.
    e_machine: u16,         // The machine type.
    e_version: u32,         // The ELF header version. Always 1.
    e_entry: u32,           // The image entry point.
    e_phoff: u32,           // The offset into the file of the program header table.
    e_shoff: u32,           // The offset into the file of the section header table.
    e_flags: u32,           
    e_ehsize: u16,          // This header's size in bytes.
    e_phentsize: u16,       // The size of each program header.
    e_phnum: u16,           // The number of entries in the program header table.
    e_shentsize: u16,       // The size of each section header.
    e_shnum: u16,           // The number of entries in the section header table.
    e_shstrndx: u16,        // The index of the section header table index of the section names table.
}

impl ELF32Header {
    fn new(file: &File) -> KernResult<ELF32Header> {
        let mut buf: [u8; 52] = [0; 52];
        try!(file.read(&mut buf, 0, 52));
        let hdr: ELF32Header = unsafe { mem::transmute(buf) };
        if hdr.ident_magic != ELF_MAGIC 
        || hdr.ident_class != ELF_CLASS_32
        || hdr.ident_data != ELF_DATA_LSB
        || hdr.ident_version != ELF_VERSION
        || hdr.e_type != ELF_TYPE_EXECUTABLE
        || hdr.e_machine != ELF_MACHINE_X86
        || hdr.e_version != ELF_VERSION as u32 {
            Err(BadImage)
        } else {
            Ok(hdr)
        }
    }
}

#[derive(Default)]
struct ELF32Section {
    sh_name: u32,       // Index into the string table of the name of the section.
    sh_type: u32,       // Section type.
    sh_flags: u32,      // Section attributes.
    sh_addr: u32,       // Address of where this section starts in virtual memory.
    sh_offset: u32,     // Byte offset from the beginning of the file of this section.
    sh_size: u32,       // The size in bytes of the section.
    sh_link: u32,       
    sh_info: u32,
    sh_addralign: u32,
    sh_entsize: u32,
}

impl ELF32Section {
    fn new(file: &File, offset: usize) -> KernResult<ELF32Section> {
        let mut buf: [u8; 40] = [0; 40];
        try!(file.read(&mut buf, offset, 40));
        Ok(unsafe { mem::transmute(buf) })
    }
    fn is_alloc(&self) -> bool {
        self.sh_flags & ELFSH_FLAGS_ALLOC != 0
    }
    fn is_writable(&self) -> bool {
        self.sh_flags & ELFSH_FLAGS_WRITE!= 0
    }   
}

struct ELF {
    file: Box<File>,
    entry: usize,
    sections: Vec<ELF32Section>,
}

impl ELF {
    fn new(file: Box<File>) -> KernResult<ELF> {
        let hdr = try!(ELF32Header::new(&*file));
        let entry = hdr.e_entry as usize;
        let mut sections = try!(Vec::new(hdr.e_shnum as usize));
        for i in 0..hdr.e_shnum as usize {
            let offset = hdr.e_shoff as usize + i * hdr.e_shentsize as usize;
            let section = try!(ELF32Section::new(&*file, offset));
            try!(sections.push(section));
        }
        Ok(ELF {
            file: file,
            entry: entry,
            sections: sections,
        })
    }
}

impl Loadable for ELF {
    
    fn load(&self, addrspace: &mut AddressSpace) -> KernResult<()> {
        for section in &self.sections {
            if section.is_alloc() {
                let addr = section.sh_addr as usize;
                let size = section.sh_size as usize;
                let offset = section.sh_offset as usize;
                let flags = if section.is_writable() { PTE_WRITABLE } else { PTE_NONE };
                let mut range = addrspace.lock_range_writer(addr, size);
                try!(range.map_all_unreserved(flags));
                try!(self.file.read(&mut range[addr..addr+size], offset, size));
            }
        }
        Ok(())
    }

    fn entry(&self) -> KernResult<usize> {
        Ok(self.entry)
    }

}
