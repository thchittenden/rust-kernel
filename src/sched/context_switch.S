// These MUST be kept consistent with src/task/thread.rs
#define TID_OFFSET 0
#define PID_OFFSET 4
#define STACK_CUR_OFFSET 8
#define STACK_TOP_OFFSET 12
#define STACK_BOTTOM_OFFSET 16

.globl context_switch
context_switch:
    // Get the current and next TCBs.
    mov 0x4(%esp), %eax
    mov 0x8(%esp), %ecx

    // Push the callee save registers.
    push %ebp
    push %ebx
    push %edi
    push %esi

    // Update the %fs descriptor so it points to the TCB and allows 
    // stack overflow detection.
    push %ecx
    call set_tl_desc
    addl $4, %esp
    
    // Reload %fs so the changes take effect.
    mov $0x28, %si
    mov %si, %fs

    // Swap the stacks.
    mov %esp, STACK_CUR_OFFSET(%eax)
    mov STACK_CUR_OFFSET(%ecx), %esp

    // Pop the callee save registers.
    pop %esi
    pop %edi
    pop %ebx
    pop %ebp
    
    // Return to the new thread.
    ret

.globl context_switch_first
context_switch_first:
    // Get the next TCB.
    mov 0x4(%esp), %ecx

    // Update the %fs descriptor so it points to the TCB and allows
    // stack overflow detection.
    push %ecx
    call set_tl_desc
    addl $4, %esp

    // Reload %fs so the changes take effect.
    mov $0x28, %si
    mov %si, %fs

    // Go to the new stack.
    mov STACK_CUR_OFFSET(%ecx), %esp

    // Pop the callee save registers. All new threads set up their stack so
    // that these registers are on them.
    pop %esi
    pop %edi
    pop %ebx
    pop %ebp

    // Return to the new thread.
    ret

set_tl_desc:
    mov 0x4(%esp), %esi
    
    // Set the first word.
    shl $16, %esi
    orl $0x14, %esi         // big enough to index stack_bottom
    mov %esi, _tl_desc

    // Set the second word.
    mov 0x4(%esp), %esi
    mov %esi, %edi
    andl $0xFF000000, %esi
    andl $0x00FF0000, %edi
    shrl $16, %edi
    orl %edi, %esi
    orl $0x00409300, %esi
    mov %esi, _tl_desc+4
    
    ret
