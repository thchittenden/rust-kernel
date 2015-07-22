ENTRY(_start)

SECTIONS
{
    /* Kernel memory begin at 1MB. */
    . = 1M;
    __kernel_start = .;

    /* put multiboot in .text since it's always located at the beginning
     * of the ELF */
    __ro_start = .;
    .text ALIGN(4K): {
        *(.multiboot)
        *(.text*)
    }
    
    .rodata : {
        *(.rodata*)
    }

    /* ramdisk stuff */ 
    __ramdisk_start = .;
    .ramdisk : {
        KEEP(*(.ramdisk))
    }
    __ramdisk_end = .;
    __ramdisk_size = __ramdisk_end - __ramdisk_start;

    __ramdisk_dir_start = .;
    .ramdisk_dir : {
        KEEP(*(.ramdisk_dir))
    }   
    __ramdisk_dir_end = .;
    __ramdisk_dir_size = __ramdisk_dir_end - __ramdisk_dir_start;

    __ro_end = .;

    /* start a new block (page) for writeable stuff */
    .data ALIGN(4K) : {
        *(.data*)
    }

    .bss : {
        *(.bss)
        *(.kernel_stack)
    }

    /DISCARD/ : {
        /**(.eh_frame)
        *(.shstrtab)
        *(.symtab)
        *(.strtab)*/
    }

    /* The heap starts after BSS. */
    __heap_start = .;

    /* The heap ends before the last page of kernel memory. This is where the head of the free 
     * frame list is. */
    . = 16M - 4K;
    __heap_end = .;
    __ffl_head = .;

    /* Kernel memory ends at 16MB. */
    . = 16M;
    __kernel_end = .;
}
