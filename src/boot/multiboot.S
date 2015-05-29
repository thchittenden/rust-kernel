#define ASM_FILE
#include "multiboot.h"

#define MULTIBOOT_HEADER_FLAGS (MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO)
#define STACK_SIZE 0x1000
#define EXCEPTION_STACK_SIZE 0x100

.globl __morestack
.globl _kernel_stack_bottom
.globl _kernel_stack_top
.globl _start
.globl _gdt
.globl _idt
.globl _tss

# Multiboot section. This contains the multiboot header that will allow 
# bootloaders to recognize this ELF and boot it.
.section .multiboot
.align MULTIBOOT_HEADER_ALIGN
    .long MULTIBOOT_HEADER_MAGIC
    .long MULTIBOOT_HEADER_FLAGS
    .long -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

# Data section containing GDT, IDT, and TSS. The IDT and GDT contain an 
# accompanying header that the "lgdt/lidt" instructions expect. These headers
# are composed of the length and address of the table.
.section .data

# The Global Descriptor Table. We create the bare minimum number of entries for
# a working kernel. Each of these segments specify at the appropriate privilege
# level the entire physical address space. This effectively "disables"
# segmentation as all segmented addresses are equivalent to their underlying
# virtual addresses. 
# We additionally create a task descriptor required to load the task register
# with the "ltr" instruction. This must be intitialized in assembly as it 
# requires arithmetic of the TSS address which cannot be performed at compile
# time.
_gdt_header:
    .word (0x8*7)
    .long _gdt
    .align 4
_gdt:
    # Empty segment       - SS 0x0000
    .long 0x00000000
    .long 0x00000000
    # Kernel code segment - SS 0x0008
    .long 0x0000FFFF
    .long 0x00CF9B00
    # Kernel data segment - SS 0x0010
    .long 0x0000FFFF
    .long 0x00CF9300
    # User code segment   - SS 0x0018
    .long 0x0000FFFF
    .long 0x00CFFB00
    # User data segment   - SS 0x0020
    .long 0x0000FFFF
    .long 0x00CFF300
_tl_desc:
    # Thread local descriptor, initialized in _start - SS 0x0028
    .long 0x00000014
    .long 0x00409300
_tss_desc:
    # TSS task descriptor, initialized in _start - SS 0x0030
    .long 0x00000000
    .long 0x00000000

# Empty IDT.
_idt_header:
    .word (0x8*256)
    .long _idt
    .align 4
_idt:
    .space (0x8*256)

# The Task-State Segment. This is required to specify an ESP0 address, which is
# the address of the stack used when entering kernel mode from user mode.
_tss:
    .space 4
_tss_esp0:
    .space 4
_tss_ss0:
    .word  0x0010 # SS0 = GDT[2]
    .space 92
    .word  0x0068 # Place the I/O bitmap past the end of the TSS.

# The thread local segment. This is to support stack overflow checks on the 
# initial stack. 
_tl_area:
    .space 16
    .long _kernel_stack_bottom

# Declare our stacks in BSS.
.section .kernel_stack,"M",@nobits,STACK_SIZE+EXCEPTION_STACK_SIZE
_kernel_stack_bottom:
    .space STACK_SIZE 
_kernel_stack_top:
_exception_stack_bottom:
    .space EXCEPTION_STACK_SIZE
_exception_stack_top:

# Text section containing all initialization/low level error handling routines.
.section .text

# This is the entry point of the kernel. In this function we perform all 
# intiailization required to get to a Rust runtime.
_start:
    # Initialize stack.
	leal _kernel_stack_top, %esp

    # Check the multiboot header magic we expect to receive.
    cmpl $MULTIBOOT_BOOTLOADER_MAGIC, %eax
    jne _start_magic_bad
    
    # Push multiboot header.
	pushl %ebx  

    # Initialize TSS descriptor. 
    leal _tss_desc, %eax
    # First word.
    leal _tss, %ebx 
    shll $16, %ebx
    orl  $0x67, %ebx        
    movl %ebx, (%eax)
    # Second word.
    leal _tss, %ebx   
    leal _tss, %ecx
    andl $0xFF000000, %ebx
    andl $0x00FF0000, %ecx
    shrl $16, %ecx
    orl  %ecx, %ebx
    orl  $0x00008900, %ebx
    movl %ebx, 4(%eax)

    # Initialize thread local descriptor.
    leal _tl_desc, %eax
    # First word.
    leal _tl_area, %ebx
    shll $16, %ebx
    orl %ebx, (%eax)
    # Second word.
    leal _tl_area, %ebx
    andl $0xFF000000, %ebx
    orl %ebx, 4(%eax)
    leal _tl_area, %ebx
    andl $0x00FF0000, %ebx
    shrl $16, %ebx
    orl %ebx, 4(%eax)
    
    # Initialize double fault handler.
    leal _idt, %eax       
    leal 0x40(%eax), %eax
    # First word.
    leal _df_handler, %ebx
    andl $0x0000FFFF, %ebx
    orl  $0x00080000, %ebx
    movl %ebx, (%eax)
    # Second word.
    leal _df_handler, %ebx 
    andl $0xFFFF0000, %ebx 
    orl  $0x00008E00, %ebx
    movl %ebx, 4(%eax) 

    # Load the GDT, IDT, and TSS. 
    lgdt _gdt_header
    lidt _idt_header
    movl $0x0030, %eax
    ltr  %ax
    
    # Initialize segment selectors.
    ljmp $0x0008, $_start_gdt_reload
_start_gdt_reload:
    movl $0x0010, %eax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %gs
    movw %ax, %ss
    movl $0x0028, %eax
    movw %ax, %fs

    # We're now ready to enter the Rust environment. Call kernel_main. 
	call kernel_main
    
    # kernel_main returned. This definitely shouldn't have happened. Notify 
    # the user and definitely halt.
    cli
    leal ret_message, %eax
    pushl %eax
    call _early_broadcast
    jmp _halt

# If the magic wasn't what we expected, notify the user and halt.
_start_magic_bad:
    leal bad_magic_message, %eax
    pushl %eax
    call _early_broadcast
    jmp _halt

# Called if a stack overflows.
__morestack:
    cli
    
    # Use the exception stack, because duh.
    leal _exception_stack_top, %esp
    pushl %ebp
    movl %esp, %ebp

    leal overflow_message, %eax
    pushl %eax
    call _early_broadcast
    jmp _halt

# If a double-fault occurs, we may be in a corrupted state. Get to a known 
# state and notify the user. 
_df_handler:
    cli
    movl $0x0010, %eax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    movl %cr0, %eax
    andl $0x7fffffff, %eax
    movl %eax, %cr0
    
    # Use the exception stack so that we preserve the main stack.
    leal _exception_stack_top, %esp
    pushl %ebp
    movl %esp, %ebp
    
    # Print the double fault message.
    leal df_message, %eax
    pushl %eax
    call _early_broadcast
    jmp _halt

# This function broadcasts a message accross the screen without relying on the
# console driver. This is used for notifying the user of an error. It could 
# probably be less annoying.
_early_broadcast:
    pushl %ebp
    movl %esp, %ebp
    pushl %esi
    pushl %edi

    # compute and push message len
    movl $-1, %ecx
    movl $0, %eax
    movl 8(%ebp), %edi
    repnz scasb
    subl 8(%ebp), %edi   
    dec  %edi    # we don't want the \0 in there
    pushl %edi

    movl $0xB8000, %edi     # start address
    movl %edi, %eax        
    addl $(2*80*25), %eax   # end address

_early_broadcast_loop:
    movl -12(%ebp), %ecx     # load message len
    movl   8(%ebp), %esi     # load message
    rep movsb                # copy over
    cmpl %edi, %eax
    jg _early_broadcast_loop

    addl $4, %esp # remove message len
    popl %edi
    popl %esi
    popl %ebp
    ret

# Stop execution permanently.
_halt:
    cli
    hlt
    jmp _halt

.section .rodata
df_message:
    .asciz "D\ro\ru\rb\rl\re\r \rF\ra\ru\rl\rt\r!\r \r"
ret_message:
    .asciz "K\re\rr\rn\re\rl\r \rR\re\rt\ru\rr\rn\re\rd\r!\r \r"
bad_magic_message:
    .asciz "B\ra\rd\r \rm\ra\rg\ri\rc\r \rv\ra\rl\ru\re\r!\r \r"
overflow_message:
    .asciz "S\rt\ra\rc\rk\r \rO\rv\re\rr\rf\rl\ro\rw\r!\r \r"
