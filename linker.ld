ENTRY(_start)

SECTIONS
{
    /* Kernel memory begin at 1MB. */
    . = 1M;
    __kernel_start = .;

    /* put multiboot in .text since it's always located at the beginning
     * of the ELF */
    .text ALIGN(4K): {
        *(.multiboot)
        *(.text*)
    }
    
    .rodata : {
        *(.rodata*)
    }

    /* start a new block (page) for writeable stuff */
    .data ALIGN(4K) : {
        *(.data*)
    }

    .bss : {
        *(.bss)
        *(.kernel_stack)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.shstrtab)
        *(.symtab)
        *(.strtab)
    }

    /* Kernel memory ends at 16 MB. */
    . = 16M;
    __kernel_end = .;
}
