#![allow(dead_code)]

// The ELF magic byte. 0x7f followed by 'ELF' in ASCII.
pub const ELF_MAGIC: u32 = 0x464c457f;
pub const ELF_CLASS_32: u8 = 0x01;
pub const ELF_CLASS_64: u8 = 0x02;
pub const ELF_DATA_LSB: u8 = 0x01;
pub const ELF_DATA_MSB: u8 = 0x02;
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

pub const ELF_TYPE_RELOCATABLE: u16 = 0x01;
pub const ELF_TYPE_EXECUTABLE: u16  = 0x02;
pub const ELF_TYPE_SHARED: u16      = 0x03;
pub const ELF_TYPE_CORE: u16        = 0x04;

pub const ELF_MACHINE_SPARC: u16   = 0x02;
pub const ELF_MACHINE_X86: u16     = 0x03;
pub const ELF_MACHINE_MIPS: u16    = 0x08;
pub const ELF_MACHINE_POWERPC: u16 = 0x14;
pub const ELF_MACHINE_ARM: u16     = 0x28;
pub const ELF_MACHINE_SUPERH: u16  = 0x2a;
pub const ELF_MACHINE_IA64: u16    = 0x32;
pub const ELF_MACHINE_X86_64: u16  = 0x3e;
pub const ELF_MACHINE_AARCH64: u16 = 0xb7;

pub const ELFSH_TYPE_NULL: u32     = 0x00;
pub const ELFSH_TYPE_PROGBITS: u32 = 0x01;
pub const ELFSH_TYPE_SYMTAB: u32   = 0x02;
pub const ELFSH_TYPE_STRTAB: u32   = 0x03;
pub const ELFSH_TYPE_RELA: u32     = 0x04;
pub const ELFSH_TYPE_HASH: u32     = 0x05;
pub const ELFSH_TYPE_DYNAMIC: u32  = 0x06;
pub const ELFSH_TYPE_NOTE: u32     = 0x07;
pub const ELFSH_TYPE_NOBITS: u32   = 0x08;
pub const ELFSH_TYPE_REL: u32      = 0x09;
pub const ELFSH_TYPE_SHLIB: u32    = 0x0a;
pub const ELFSH_TYPE_DYNSYM: u32   = 0x0b;
pub const ELFSH_TYPE_LOPROC: u32   = 0x70000000;
pub const ELFSH_TYPE_HIPROC: u32   = 0x7fffffff;
pub const ELFSH_TYPE_LOUSER: u32   = 0x80000000;
pub const ELFSH_TYPE_HIUSER: u32   = 0xffffffff;

pub const ELFSH_FLAGS_WRITE: u32     = 0x1;
pub const ELFSH_FLAGS_ALLOC: u32     = 0x2;
pub const ELFSH_FLAGS_EXECINSTR: u32 = 0x4;
pub const ELFSH_FLAGS_MASKPROC: u32  = 0xf0000000;





