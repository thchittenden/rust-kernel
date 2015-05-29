use alloc::boxed::Box;

pub trait HasNode<T> {
    fn get_node(&self) -> &Node<T>;
    fn get_node_mut(&mut self) -> &mut Node<T>;
}

pub struct Node<T> {
    next: Box<T>,
    prev: *const T,
}
