// This is a naive allocator that is very good at allocating and very bad at 
// deallocating.
use core::prelude::*;
use core::ptr::Unique;
use core::mem;
use ::Allocator;
logger_init!(Trace);

pub struct NaiveAllocator {
    heap_start: usize,
    heap_end: usize,
    heap_cur: usize,
}

pub const NAIVE_ALLOCATOR_INIT: NaiveAllocator = NaiveAllocator {
    heap_start: 0,
    heap_end: 0,
    heap_cur: 0,
};

impl NaiveAllocator {
    
    pub fn init(&mut self, heap_start: usize, heap_end: usize) {
        assert_eq!(self.heap_start, 0);
        assert_eq!(self.heap_end, 0);
        assert_eq!(self.heap_cur, 0);
        assert!(heap_start < heap_end);
        self.heap_start = heap_start;
        self.heap_cur = heap_start;
        self.heap_end = heap_end;
    }

}

impl Allocator for NaiveAllocator {

    fn allocate<T>(&mut self, elem: T) -> Option<Unique<T>> {
        let size = mem::size_of::<T>();
        trace!("trying to allocate {} bytes", size);
        if self.heap_cur + size > self.heap_end {
            trace!("not enough space on heap");
            None
        } else {
            trace!("allocated {} bytes at {:x}", size, self.heap_cur);
            // Allocate some space and copy the data in.
            let alloc = unsafe { Unique::new(self.heap_cur as *mut T) };
            unsafe { **alloc = elem };
            self.heap_cur += size;
            Some(alloc)
        }
    }

    fn deallocate<T>(&mut self, elem: Unique<T>) {
        // Welp.
    }

}
