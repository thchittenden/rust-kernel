#define PUSHA_OFFSET 0
#define IRET_OFFSET 32
#define INT_DISPATCH rust_interrupt_dispatch

#define SYS_WRAPPER(id, name) .globl _isr_wrapper_ ## name; \
        _isr_wrapper_ ## name: \
        pushl $0;       \
        pusha;          \
        leal PUSHA_OFFSET(%esp), %eax;  \
        leal IRET_OFFSET(%esp), %ebx;   \
        pushl %eax;     \
        pushl %ebx;     \
        pushl $id;      \
        call INT_DISPATCH;   \
        addl $12, %esp; \
        popa;           \
        addl $4, %esp;  \
        iret;    

#define EXN_WRAPPER(id, name) .globl _isr_wrapper_ ## name; \
        _isr_wrapper_ ## name: \
        pusha;          \
        leal PUSHA_OFFSET(%esp), %eax;  \
        leal IRET_OFFSET(%esp), %ebx;   \
        pushl %eax;     \
        pushl %ebx;     \
        pushl $id;      \
        call INT_DISPATCH;   \
        addl $12, %esp; \
        popa;           \
        addl $4, %esp;  \
        iret;    

SYS_WRAPPER(0, DIVIDE_ERROR)

SYS_WRAPPER(2, NMI)
SYS_WRAPPER(3, BREAKPOINT)
SYS_WRAPPER(4, OVERFLOW)
SYS_WRAPPER(5, BOUND)
SYS_WRAPPER(6, INV_OPCODE)
SYS_WRAPPER(7, NO_MATH)

SYS_WRAPPER(9, COPROC_OVERRUN)
EXN_WRAPPER(10, INVALID_TSS)
EXN_WRAPPER(11, NOT_PRESENT)
EXN_WRAPPER(12, STACK_SEG_FAULT)
EXN_WRAPPER(13, PROTECTION_FAULT)
EXN_WRAPPER(14, PAGE_FAULT)

SYS_WRAPPER(16, MATH_FAULT)
EXN_WRAPPER(17, ALIGNMENT_FAULT)
SYS_WRAPPER(18, MACHINE_CHECK)
SYS_WRAPPER(19, SIMD_FAULT)
SYS_WRAPPER(20, VIRT_FAULT)

SYS_WRAPPER(32, TIMER_INT)
SYS_WRAPPER(33, KEYBOARD_INT)
SYS_WRAPPER(35, SERIAL24_INT)
SYS_WRAPPER(36, SERIAL13_INT)
SYS_WRAPPER(37, PARALLEL2_INT)
SYS_WRAPPER(38, FLOPPY_INT)
SYS_WRAPPER(39, PARALLEL1_INT)
SYS_WRAPPER(40, RTC_INT)
SYS_WRAPPER(41, ACPI_INT)
SYS_WRAPPER(42, UNUSED1_INT)
SYS_WRAPPER(43, UNUSED2_INT)
SYS_WRAPPER(44, PS2_INT)
SYS_WRAPPER(45, FPU_INT)
SYS_WRAPPER(46, PRIMARY_ATA_INT)
SYS_WRAPPER(47, SECONDARY_ATA_INT)

