/// A simple hash map implementation based on separate chaining. 
///
/// We chose to use separate chaining as insertions can't fail as they can with something like
/// open-addressing. A consequence of this however is that keys must always be derivable from their
/// values. 
///
/// Currently uses lookup_{or_insert}_{mut} for variations of looking up but I would like to use
/// the more ergonomic Entry method used in libstd. I'm not sure if this makes sense for separate
/// chaining however.
///
use alloc::boxed::Box;
use core::prelude::*;
use core::hash::{Hash, Hasher};
use core::marker;
use dynarray::{self, DynArray};
use link::HasSingleLink;
use slist::{self, SList};

/// We assume the hash function is uniformly distributed in the lowest bits so that this doesn't
/// result in horrible collisions. Otherwise this should be prime!
const DEFAULT_SIZE: usize = 16;
const FNV_BASE: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub trait HasKey<K> {
    fn get_key(&self) -> &K;
}

pub struct HashMap<K: Hash + Eq, V: HasKey<K> + HasSingleLink<V> + ?Sized> {
    count: usize,
    table: DynArray<SList<V>>,
    _marker: marker::PhantomData<K>,
}

impl<K: Hash + Eq, V: HasKey<K> + HasSingleLink<V> + ?Sized> HashMap<K, V> {
    
    fn hash(&self, key: &K) -> u64 {
        let mut state = FNVHasher::new();
        key.hash(&mut state);
        state.finish()
    }

    fn entry(&self, key: &K) -> usize {
        self.hash(key) as usize % self.table.len()
    }

    /// Attempts to construct a new hashmap.
    pub fn new() -> Option<HashMap<K, V>> {
        DynArray::new(DEFAULT_SIZE).map(|array| {
            HashMap {
                count: 0,
                table: array,
                _marker: marker::PhantomData
            }
        })
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Inserts a new entry into the hash map and returns the evicted value if there was one.
    pub fn insert(&mut self, val: Box<V>) -> Option<Box<V>> {
        let (res, entry) = {
            let key = val.get_key();
            let entry = self.entry(key);
            let res = self.remove(key);
            (res, entry)
        };
        self.count += 1;
        self.table[entry].push(val);
        res
    }

    /// Returns whether or not an element with the given key is in the map.
    pub fn contains(&self, key: &K) -> bool {
        self.lookup(key).is_some()
    }

    /// Tries to remove an element with the given key.
    pub fn remove(&mut self, key: &K) -> Option<Box<V>> {
        let entry = self.entry(key);
        let res = self.table[entry].remove_where(|elem| elem.get_key() == key);
        if res.is_some() {
            self.count -= 1; 
        }
        res
    }

    /// Tries to borrow an element with the given key.
    pub fn lookup(&self, key: &K) -> Option<&V> {
        let entry = self.entry(key);
        self.table[entry].borrow_where(|elem| elem.get_key() == key)
    }

    /// Tries to mutably borrow an element with the given key.
    pub fn lookup_mut(&mut self, key: &K) -> Option<&mut V> {
        let entry = self.entry(key);
        self.table[entry].borrow_mut_where(|elem| elem.get_key() == key)
    }

    /// Tries to lookup an element in the map and if it is not present, inserts an element. Returns
    /// a reference to the element now in the map.
    pub fn lookup_or_insert<F>(&mut self, key: &K, new: F) -> &V where F: FnOnce() -> Box<V> {
        if !self.contains(key) {
            self.insert(new());
        }
        self.lookup(key).unwrap()
    }

    /// Tries to lookup an element in the map and if it is not present, inserts an element. Returns
    /// a mutable reference to the element now in the map.
    pub fn lookup_or_insert_mut<F>(&mut self, key: &K, new: F) -> &mut V where F: FnOnce() -> Box<V> {
        if !self.contains(key) {
            self.insert(new());
        }
        self.lookup_mut(key).unwrap()
    }

    pub fn iter_keys(&self) -> KeyIter<K, V> {
        unimplemented!()
    }
}

pub struct ValueIter<'a, K: 'a + Eq + Hash, V: 'a + HasKey<K> + HasSingleLink<V> + ?Sized> {
    table_iter: dynarray::Iter<'a, SList<V>>,
    entry_iter: slist::Iter<'a, V>,
    _marker: marker::PhantomData<&'a K>,
}

impl<'a, K: Eq + Hash, V: HasKey<K> + HasSingleLink<V>> Iterator for ValueIter<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        self.entry_iter.next().or_else(|| {
            self.entry_iter = try_op!(self.table_iter.next()).into_iter();
            self.entry_iter.next()
        })
    }
}

pub struct KeyIter<'a, K: 'a + Eq + Hash, V: 'a + HasKey<K> + HasSingleLink<V> + ?Sized> {
    value_iter: ValueIter<'a, K, V>
}

impl<'a, K: Eq + Hash, V: HasKey<K> + HasSingleLink<V>> Iterator for KeyIter<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<&'a K> {
        self.value_iter.next().map(|v| v.get_key())
    }
}


pub struct FNVHasher {
    accum: u64
}

impl FNVHasher {
    pub fn new () -> FNVHasher {
        FNVHasher { accum: FNV_BASE }
    }
}

impl Hasher for FNVHasher {
    fn finish(&self) -> u64 {
        self.accum
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.accum *= FNV_PRIME;
            self.accum ^= *byte as u64;
        }
    }
}

