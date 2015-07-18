.globl print
print:
    push %ebp
    movl %esp, %ebp
    mov 8(%ebp), %eax
    int $0x80
    pop %ebp
    ret
