/// A simple hash map implementation based on separate chaining. 
///
/// We chose to use separate chaining as insertions can't fail as they can with something
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
use dynarray::DynArray;
use link::HasSingleLink;

/// We assume the hash function is uniformly distributed in the lowest bits so that this doesn't
/// result in horrible collisions. Otherwise this should be prime!
const DEFAULT_SIZE: usize = 16;
const FNV_BASE: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

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

pub struct HashMap<K: Hash + Eq, V: HasSingleLink<V>> {
    count: usize,
    keygen: fn(&V) -> &K,
    table: DynArray<Option<Box<V>>>
}

impl<K: Hash + Eq, V: HasSingleLink<V>> HashMap<K, V> {

    fn hash(&self, key: &K) -> u64 {
        let mut state = FNVHasher::new();
        key.hash(&mut state);
        state.finish()
    }

    fn keygen<'a>(&self, val: &'a V) -> &'a K {
        let keygen = self.keygen;
        keygen(val)
    }

    pub fn new(keygen: fn(&V) -> &K) -> Option<HashMap<K, V>> {
        DynArray::new(DEFAULT_SIZE).map(|array| {
            HashMap {
                count: 0,
                keygen: keygen,
                table: array
            }
        })
    }

    pub fn insert(&mut self, mut val: V) -> Option<V> {
        let (res, entry) = {
            let key = self.keygen(&val);
            let entry = self.hash(key) as usize % self.table.len();
            let res = self.remove(key);
            (res, entry)
        };
        match self.table[entry].take() {
            Some(v) =>  {
                // There are other elements in this chain. Push this one to the front.
                val.slink_mut().link = Some(v);
                self.table[entry] = Some(val);
            }
            None => { 
                // This is the only element in this chain.
                self.table[entry] = Some(val) 
            }
        }
        res
    }

    pub fn contains(&self, key: &K) -> bool {
        unimplemented!()
    }

    pub fn remove(&self, key: &K) -> Option<Box<V>> {
        unimplemented!()
    }

    pub fn lookup(&self, key: &K) -> Option<&V> {
        unimplemented!()
    }

    pub fn lookup_mut(&self, key: &K) -> Option<&mut V> {
        unimplemented!()
    }

    pub fn lookup_or_insert<F>(&self, key: K, new: F) -> &V where F: FnOnce() -> V {
        unimplemented!()
    }

    pub fn lookup_or_insert_mut<F>(&mut self, key: K, new: F) -> &mut V where F: FnOnce() -> V {
        unimplemented!()
    }
}


