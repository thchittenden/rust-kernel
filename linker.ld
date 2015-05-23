ENTRY(_start)

SECTIONS
{
    /* begin at 1MiB */
    . = 1M;
	__kernel_start = .;

	/* put multiboot in .text since it's always loaded first */
    .text BLOCK(4K) : {
		*(.multiboot)
        *(.text*)
    }
	
    .rodata : {
        *(.rodata*)
    }

    /* start a new block (page) for writeable stuff */
    .data BLOCK(4K) : {
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

	__kernel_end = ALIGN(4K);
}
