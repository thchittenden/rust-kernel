use core::prelude::*;
use alloc::boxed::Box;
use collections::hashmap::{HasKey, HashMap};
use collections::link::{DoubleLink, HasDoubleLink};
logger_init!(Trace);

#[derive(Default)]
struct X {
    key: usize,
    val: usize,
    node: DoubleLink<X>
}

impl HasKey<usize> for X { 
    fn get_key(&self) -> &usize {
        &self.key
    }
}

impl HasDoubleLink<X> for X {
    fn dlink(&self) -> &DoubleLink<X> { &self.node }
    fn dlink_mut(&mut self) -> &mut DoubleLink<X> { &mut self.node }
}

pub fn test() {
    trace!("\ntesting map");
    let mut map: HashMap<usize, X> = HashMap::new().unwrap();
    let x = Box::new(X { key: 3, val: 4, node: DoubleLink::default() }).unwrap();
    let y = Box::new(X { key: 4, val: 5, node: DoubleLink::default() }).unwrap();
    let z = Box::new(X { key: 7, val: 9, node: DoubleLink::default() }).unwrap();
    map.insert(x);
    map.insert(y);
    map.insert(z);

    assert!(map.count() == 3);
    assert!(map.lookup(&3).unwrap().val == 4);
    assert!(map.count() == 3);
    
    map.lookup_mut(&4).unwrap().val = 42;
    assert!(map.lookup(&4).unwrap().val == 42);

    assert!(map.remove(&3).unwrap().val == 4);
    assert!(map.remove(&4).unwrap().val == 42);
    assert!(map.count() == 1);

    assert!(map.remove(&7).unwrap().val == 9);
    assert!(map.count() == 0);
}

