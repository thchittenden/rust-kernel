//!
//! This module contains the definition of the `Node` object used by all collections.
//!
//! This `Node` owns the pointer to the next object thus objects can only be in one queue at a
//! time. This is additionally constrained by the fact that HasNode can only return a single Node
//! which further enforces being in only a single queue. 
//!
use core::option::Option;
use alloc::boxed::Box;
use raw::Raw;

pub trait HasNode<T> {
    fn node(&self) -> &Node<T>;
    fn node_mut(&mut self) -> &mut Node<T>;
}

pub struct Node<T> {
    pub next: Option<Box<T>>,
    pub prev: Option<Raw<T>>,
}

