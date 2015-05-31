// These MUST be kept consistent with src/task/thread.rs
#define TID_OFFSET 0
#define PID_OFFSET 4
#define STACK_CUR_OFFSET 8
#define STACK_TOP_OFFSET 12
#define STACK_BOTTOM_OFFSET 16

.globl context_switch
context_switch:
    push %ebp
    mov %esp, %ebp

    // Get the current and next TCBs.
    mov 0x8(%ebp), %eax
    mov 0xc(%ebp), %ecx

    // Push the callee save registers.
    push %ebx
    push %edi
    push %esi

    // Update the %fs descriptor so it points to the TCB and allows 
    // stack overflow detection.
    push %ecx
    call set_tl_desc
    pop %ecx

    // Swap the stacks.
    mov %esp, STACK_CUR_OFFSET(%eax)
    mov STACK_CUR_OFFSET(%ecx), %esp

    // Pop the callee save registers.
    pop %esi
    pop %edi
    pop %ebx
    
    // Return to the new thread.
    leave
    ret

.globl context_switch_first
context_switch_first:
    push %ebp
    mov %esp, %ebp

    // Get the next TCB.
    mov 0x8(%ebp), %ecx

    // Update the %fs descriptor so it points to the TCB and allows
    // stack overflow detection.
    push %ecx
    call set_tl_desc
    pop %ecx

    // Go to the new stack.
    mov STACK_CUR_OFFSET(%ecx), %esp

    // Pop the callee save registers. All new threads set up their stack so
    // that these registers are on them.
    pop %esi
    pop %edi
    pop %ebx

    // Return to the new thread.
    leave
    ret

set_tl_desc:
    mov 0x4(%esp), %ecx
    
    // Set the first word.
    shl $16, %ecx
    orl $0x14, %ecx         // big enough to index stack_bottom
    mov %ecx, _tl_desc

    // Set the second word.
    mov 0x4(%esp), %ecx
    mov %ecx, %ebx
    andl $0xFF000000, %ecx
    andl $0x00FF0000, %ebx
    shrl $16, %ebx
    orl %ebx, %ecx
    orl $0x00409300, %ecx
    mov %ecx, _tl_desc+4
    
    ret
