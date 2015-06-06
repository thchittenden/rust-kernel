/// A simple hash map implementation based on separate chaining. 
///
/// Currently uses lookup_{or_insert}_{mut} for variations of looking up but I would like to use
/// the more ergonomic Entry method used in libstd.
///
use core::prelude::*;

pub struct HashMap<K,V> {
    x: Option<(K, V)>
}

impl<K, V> HashMap<K, V> {

    pub fn new() -> HashMap<K, V> {
        HashMap {
            x: None
        }
    }

    pub fn insert(&self, key: K, val: V) {
        unimplemented!()   
    }

    pub fn contains(&self, key: K) -> bool {
        unimplemented!()
    }

    pub fn remove(&self, key: K, val: V) {
        unimplemented!()
    }

    pub fn lookup(&self, key: K) -> Option<&V> {
        unimplemented!()
    }

    pub fn lookup_mut(&self, key: K) -> Option<&mut V> {
        unimplemented!()
    }

    pub fn lookup_or_insert<F>(&self, key: K, new: F) -> &V where F: FnOnce() -> V {
        unimplemented!()
    }

    pub fn lookup_or_insert_mut<F>(&mut self, key: K, new: F) -> &mut V where F: FnOnce() -> V {
        unimplemented!()
    }
}


