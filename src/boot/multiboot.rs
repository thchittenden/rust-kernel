#[packed]
pub struct MultibootHeader {
    flags: u32,

    // only defined if flags[0] set
    mem_lower: u32,
    mem_upper: u32,

    // only defined if flags[1] set
    boot_drive: u8,
    boot_part1: u8,
    boot_part2: u8,
    boot_part3: u8,

    // only defined if flags[2] set
    cmdline: *const u8,

    // only defined if flags[3] set
    mods_count: u32,
    mods_addr:  u32,

    // only defined if flags[5] set (ignoring flags[4], only for a.out kernels)
    syms_num:   u32,
    syms_size:  u32,
    syms_addr:  u32,
    syms_shndx: u32,

    // only defined if flags[6] set
    mmap_length: u32,
    mmap_addr:   u32,

    // only defined if flags[7] set
    drives_length: u32,
    drives_addr:   u32,

    // only defined if flags[8] set
    config_table: u32,

    // only defined if flags[9] set
    boot_loader_name: *const u8,

    // only defined if flags[10] set
    apm_table: u32,

    // only defined if flags[11] set
    vbe_control_info:  u32,
    vbe_mode_info:     u32,
    vbe_mode:          u32,
    vbe_interface_seg: u32,
    vbe_interface_off: u32,
    vbe_interface_len: u32,

}
