use core::prelude::*;
use alloc::boxed::Box;
use collections::hashmap::HashMap;
use collections::link::{DoubleLink, HasDoubleLink};
logger_init!(Trace);

#[derive(Default)]
struct X {
    key: usize,
    val: usize,
    node: DoubleLink<X>
}

impl X { 
    pub fn getkey(&self) -> &usize {
        &self.key
    }
}

impl HasDoubleLink for X {
    type T = X;
    fn dlink(&self) -> &DoubleLink<X> { &self.node }
    fn dlink_mut(&mut self) -> &mut DoubleLink<X> { &mut self.node }
}

pub fn test() {
    trace!("\ntesting map");
    let mut map: HashMap<usize, X> = HashMap::new(X::getkey).unwrap();
    let x = Box::new(X { key: 3, val: 4, node: DoubleLink::default() }).unwrap();
    let y = Box::new(X { key: 4, val: 5, node: DoubleLink::default() }).unwrap();
    map.insert(x);
    map.insert(y);
}

