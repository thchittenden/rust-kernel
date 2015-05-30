use util::asm;

const PIC_IRQ_BASE: u8 = 32;
const PIC_IRQ_MASTER_BASE: u8 = PIC_IRQ_BASE;
const PIC_IRQ_SLAVE_BASE: u8 = PIC_IRQ_BASE + 8;
const PIC_IRQ_COUNT: u8 = 16;

const MASTER_PIC_COMM: u16 = 0x0020;
const MASTER_PIC_DATA: u16 = 0x0021;
const SLAVE_PIC_COMM: u16 = 0x00a0;
const SLAVE_PIC_DATA: u16 = 0x00a1;

const PIC_EOI: u8 = 0x20;

const ICW1_ICW4: u8     = 0x01;
const ICW1_SINGLE: u8   = 0x02;
const ICW1_INTERVAL: u8 = 0x04;
const ICW1_LEVEL: u8    = 0x08;
const ICW1_INIT: u8     = 0x10;
const ICW4_8086: u8     = 0x01;
const ICW4_AUTO: u8     = 0x02;
const ICW4_BUF_SLAVE: u8  = 0x08;
const ICW4_BUF_MASTER: u8 = 0x0c;
const ICW4_SFNM: u8     = 0x10;

pub fn init_pic() {
    // Start the initialization sequence.
    asm::outb8(MASTER_PIC_COMM, ICW1_INIT | ICW1_ICW4);
    asm::outb8(SLAVE_PIC_COMM,  ICW1_INIT | ICW1_ICW4);
    
    // Set the PIC vector offsets.
    asm::outb8(MASTER_PIC_DATA, PIC_IRQ_MASTER_BASE);
    asm::outb8(SLAVE_PIC_DATA, PIC_IRQ_SLAVE_BASE);

    // Tell the master and slave PICs where they are located.
    asm::outb8(MASTER_PIC_DATA, 4);
    asm::outb8(SLAVE_PIC_DATA, 2);

    // Set the PICs to 8086 mode.
    asm::outb8(MASTER_PIC_DATA, ICW4_8086);
    asm::outb8(SLAVE_PIC_DATA, ICW4_8086);

    // Acknowledge any outsanding IRQs.
    asm::outb8(MASTER_PIC_COMM, PIC_EOI);
    asm::outb8(SLAVE_PIC_COMM, PIC_EOI);

    // Enable all IRQs on the master and the slave.
    asm::outb8(MASTER_PIC_DATA, 0);
    asm::outb8(SLAVE_PIC_DATA, 0);
}

pub fn acknowledge_irq(irq: u8) {
    assert!(PIC_IRQ_BASE <= irq && irq < PIC_IRQ_BASE + PIC_IRQ_COUNT);
    let pic_irq = irq - PIC_IRQ_BASE;
    if pic_irq < 8 {
        // ACK master.
        asm::outb8(MASTER_PIC_COMM, PIC_EOI);
    } else {
        // ACK slave.
        asm::outb8(SLAVE_PIC_COMM, PIC_EOI);

    }
}

