#define ASM_FILE
#include "multiboot.h"

#define MULTIBOOT_HEADER_FLAGS (MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO)
#define STACK_SIZE 0x1000

.globl _kernel_stack_bottom
.globl _kernel_stack_top
.globl _start
.globl _gdt
.globl _idt
.globl _tss

# multiboot section
.section .multiboot
.align MULTIBOOT_HEADER_ALIGN
    .long MULTIBOOT_HEADER_MAGIC
    .long MULTIBOOT_HEADER_FLAGS
    .long -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

# data section containing GDT, IDT, and TSS
.section .data
_gdt_header:
    .word (0x8*6)
    .long _gdt
    .align 4
_gdt:
    # empty segment       - SS 0x0000
    .long 0x00000000
    .long 0x00000000
    # kernel code segment - SS 0x0008
    .long 0x0000FFFF
    .long 0x00CF9B00
    # kernel data segment - SS 0x0010
    .long 0x0000FFFF
    .long 0x00CF9300
    # user code segment   - SS 0x0018
    .long 0x0000FFFF
    .long 0x00CFFB00
    # user data segment   - SS 0x0020
    .long 0x0000FFFF
    .long 0x00CFF300
_tss_desc:
    # tss task descriptor, initialized in _start - SS 0x0028
    .long 0x00000000
    .long 0x00000000

# Empty IDT.
_idt_header:
    .word (0x8*256)
    .long _idt
    .align 4
_idt:
    .space (0x8*256)

_tss:
    .space 8
    .word  0x0010 # SS0 = GDT[2]
    .space 92
    .word  0x0068 # don't use io-bitmap

# 4K kernel stack.
.section .kernel_stack,"M",@nobits,STACK_SIZE
_kernel_stack_bottom:
    .space STACK_SIZE 
_kernel_stack_top:

# Text section with initialization routines.
.section .text
_start:
    # Initialize stack.
	leal _kernel_stack_top, %esp

    # check the multiboot header magic we expect to receive
    cmpl $MULTIBOOT_BOOTLOADER_MAGIC, %eax
    je _start_magic_good
_start_magic_bad:
    leal bad_magic_message, %eax
    pushl %eax
    call scream_msg
_start_magic_bad_loop:
    jmp _start_magic_bad_loop

_start_magic_good:
    # Push multiboot header.
	pushl %ebx  

    # Initialize TSS descriptor.
    leal _tss_desc, %eax  # get tss descriptor entry
    leal _tss, %ebx       # construct and set first byte
    shll $16, %ebx
    orl  $0x67, %ebx        
    movl %ebx, (%eax)
    
    leal _tss, %ebx       # construct and set second byte
    leal _tss, %ecx
    andl $0xFF000000, %ebx
    andl $0x00FF0000, %ecx
    shrl $16, %ecx
    orl  %ecx, %ebx
    orl  $0x00008900, %ebx
    movl %ebx, 4(%eax)
    
#if 0 
    # initialize double fault handler
    leal _idt, %eax        # get df idt entry
    leal 0x40(%eax), %eax
    
    leal df_handler, %ebx # construct and set first byte
    andl $0x0000FFFF, %ebx
    orl  $0x00080000, %ebx
    movl %ebx, (%eax)

    leal df_handler, %ebx # construct and set second byte
    andl $0xFFFF0000, %ebx 
    orl  $0x00008E00, %ebx
    movl %ebx, 4(%eax) 
#endif

    # load gdt, idt, and tss
    lgdt _gdt_header
    lidt _idt_header
    
    movl $0x0028, %eax
    ltr  %ax
    
    # initialize segment selectors
    ljmp $0x0008, $_start_gdt_reload
_start_gdt_reload:
    movl $0x0010, %eax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss

    # call bootstrap routine
	call kernel_main
    
    # uh oh, we shouldn't have returned, notify user and halt for eternity
    cli
    leal ret_message, %eax
    pushl %eax
    call scream_msg
_end:
    cli
	hlt
    jmp _end

# Shamelessly copied from 410.
df_handler:
    cli
    movl $0x0010, %eax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    # Allocate ourselves a small stack.
    leal _kernel_stack_bottom, %esp
    addl $0x100, %esp
    
    # Print the double fault message.
    leal df_message, %eax
    pushl %eax
    call scream_msg
df_handler_loop:
    hlt
    jmp df_handler_loop

# void scream_msg(char* str)
scream_msg:
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

scream_msg_loop:
    movl -12(%ebp), %ecx     # load message len
    movl   8(%ebp), %esi     # load message
    rep movsb               # copy over
    cmpl %edi, %eax
    jg scream_msg_loop

    addl $4, %esp # remove message len
    popl %edi
    popl %esi
    popl %ebp
    ret

.section .rodata
df_message:
    .asciz "D\ro\ru\rb\rl\re\r \rF\ra\ru\rl\rt\r!\r \r"
ret_message:
    .asciz "K\re\rr\rn\re\rl\r \rR\re\rt\ru\rr\rn\re\rd\r!\r \r"
bad_magic_message:
    .asciz "B\ra\rd\r \rm\ra\rg\ri\rc\r \rv\ra\rl\ru\re\r!\r \r"
