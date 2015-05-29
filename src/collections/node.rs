//!
//! This module contains the definition of the `Node` object used by all collections.
//!
//! This `Node` owns the pointer to the next object thus objects can only be in one queue at a
//! time. This is additionally constrained by the fact that HasNode can only return a single Node
//! which further enforces being in only a single queue. 
//!
use alloc::boxed::Box;

pub trait HasNode<T> {
    fn get_node(&self) -> &Node<T>;
    fn get_node_mut(&mut self) -> &mut Node<T>;
}

pub struct Node<T> {
    next: Box<T>,
    prev: *const T,
}
