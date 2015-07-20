// The ELF magic byte. 0x7f followed by 'ELF' in ASCII.
pub const ELF_MAGIC: u32 = 0x464c457f;
pub const ELF_CLASS_32: u8 = 0x01;
pub const ELF_CLASS_64: u8 = 0x02;
pub const ELF_VERSION: u8 = 0x01;

pub const ELF_OSABI_SYSTEMV: u8 = 0x00;
pub const ELF_OSABI_HPUX: u8    = 0x01;
pub const ELF_OSABI_NETBSD: u8  = 0x02;
pub const ELF_OSABI_LINUX: u8   = 0x03;
pub const ELF_OSABI_SOLARIS: u8 = 0x06;
pub const ELF_OSABI_AIX: u8     = 0x07;
pub const ELF_OSABI_IRIX: u8    = 0x08;
pub const ELF_OSABI_FREEBSD: u8 = 0x09;
pub const ELF_OSABI_OPENBSD: u8 = 0x0c;
pub const ELF_OSABI_OPENVMS: u8 = 0x0d; 

pub const ELF_TYPE_RELOCATABLE: u8 = 0x01;
pub const ELF_TYPE_EXECUTABLE: u8  = 0x02;
pub const ELF_TYPE_SHARED: u8      = 0x03;
pub const ELF_TYPE_CORE: u8        = 0x04;

pub const ELF_MACHINE_SPARC: u8   = 0x02;
pub const ELF_MACHINE_X86: u8     = 0x03;
pub const ELF_MACHINE_MIPS: u8    = 0x08;
pub const ELF_MACHINE_POWERPC: u8 = 0x14;
pub const ELF_MACHINE_ARM: u8     = 0x28;
pub const ELF_MACHINE_SUPERH: u8  = 0x2a;
pub const ELF_MACHINE_IA64: u8    = 0x32;
pub const ELF_MACHINE_X86_64: u8  = 0x3e;
pub const ELF_MACHINE_AARCH64: u8 = 0xb7;

pub struct ELF32Header {
    magic: u32,
    class: u8,
    data: u8,
    version0: u8,
    abi_os: u8,
    abi_version: u8,
    reserved0: u8,
    elf_type: u8,
    machine: u8,
    version1: u8,
    entry: u32,
    program_header_table: u32,
    section_header_table: u32,
    flags: u32,
    header_size: u16,
    phent_size: u16,
    phnum: u16,
    shent_size: u16,
    shnum: u16,
    shstrndx: u16,
}

