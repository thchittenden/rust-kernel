.globl vanish
vanish:
    push %ebp
    movl %esp, %ebp
    mov 8(%ebp), %eax
    int $0x81
    pop %ebp
    ret
