// This is a naive allocator that is very good at allocating and very bad at 
// deallocating.
use core::prelude::*;
use core::ptr::Unique;
use core::mem;
use Allocator;
use util::align_up;
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

    fn allocate_raw(&mut self, size: usize, align: usize) -> Option<usize> {
        trace!("trying to allocate {} bytes aligned to 0x{:x}", size, align);
        if align_up(self.heap_cur, align) + size > self.heap_end {
            trace!("not enough space on heap");
            None
        } else {
            trace!("allocated {} bytes at {:x}", size, align_up(self.heap_cur, align));
            self.heap_cur = align_up(self.heap_cur, align) + size;
            Some(self.heap_cur - size)
        }
    }

    fn deallocate_raw(&mut self, addr: usize, size: usize) {
        // Welp.
    }

}
