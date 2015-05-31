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

    /* Kernel memory ends at 16 MB. */
    . = 16M;
    __heap_end = .;
    __kernel_end = .;
}
