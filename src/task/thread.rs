use collections::node::{Node, HasNode};

#[repr(C, packed)]
pub struct Thread {
    
    tid: i32,
    pid: i32,
    stack_cur: usize, 
    stack_top: usize,
    stack_bottom: usize, // This MUST be at offset 0x10
    sched_node: Node<Thread>
}

impl HasNode<Thread> for Thread {
    fn get_node(&self) -> &Node<Thread> {
        &self.sched_node
    }
    fn get_node_mut(&mut self) -> &mut Node<Thread> {
        &mut self.sched_node
    }
}
