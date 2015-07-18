ENTRY(_start)

SECTIONS
{
    /* User memory begin at 16MB. */
    . = 16M;

    .text ALIGN(4K): {
        *(.text*)
    }
    
    .rodata : {
        *(.rodata*)
    }

    .data ALIGN(4K) : {
        *(.data*)
    }

    .bss : {
        *(.bss)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.shstrtab)
        *(.symtab)
        *(.strtab)
        *(.note.*)
    }

}
