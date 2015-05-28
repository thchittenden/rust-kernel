#![crate_name="alloc"]
#![crate_type="rlib"]
#![feature(no_std,lang_items,unique,core,unsafe_no_drop_flag)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate sync;
extern crate console;
logger_init!(Trace);

pub mod boxed;

mod naive;
mod lmm;

use core::prelude::*;
use core::mem;
use core::mem::min_align_of;
use core::ptr;
use core::ptr::Unique;
use sync::mutex::Mutex;
use lmm::{LMMAllocator, LMM_ALLOCATOR_INIT};

trait Allocator {

    fn allocate_raw(&mut self, size: usize, align: usize) -> Option<usize>;

    fn deallocate_raw(&mut self, addr: usize, size: usize);

    fn allocate_aligned<T>(&mut self, elem: T, align: usize) -> Option<Unique<T>> {
        let alloc = self.allocate_raw(mem::size_of::<T>(), align);
        alloc.map(|addr| {
            let uniq = unsafe { Unique::new(addr as *mut T) };
            unsafe { **uniq = elem }
            uniq
        })
    }
    
    fn allocate<T>(&mut self, elem: T) -> Option<Unique<T>> {
        self.allocate_aligned(elem, min_align_of::<T>())
    }

    // Drops the contents of the box and deallocates its memory. I'm not sure
    // the semantics of ptr::read, but if it reads the contents of the pointer
    // onto the stack, this is probably suboptimal.
    fn deallocate<T>(&mut self, mut elem: Unique<T>) {
        let addr = unsafe { elem.get_mut() } as *mut T as usize;
        let size = mem::size_of::<T>();
        unsafe { drop(ptr::read(elem.get_mut() as *mut T)) };
        self.deallocate_raw(addr, size);
    }

}

static ALLOCATOR: Mutex<LMMAllocator> = static_mutex!(LMM_ALLOCATOR_INIT);

pub fn init() {
    let heap_start = linker_sym!(__heap_start);
    let heap_end = linker_sym!(__heap_end);
    ALLOCATOR.lock().unwrap().init(heap_start, heap_end);
}

pub fn allocate<T>(elem: T) -> Option<Unique<T>> {
    ALLOCATOR.lock().unwrap().allocate(elem)
}

pub fn allocate_aligned<T>(elem: T, align: usize) -> Option<Unique<T>> {
    ALLOCATOR.lock().unwrap().allocate_aligned(elem, align)
}

pub fn deallocate<T>(elem: Unique<T>) {
    ALLOCATOR.lock().unwrap().deallocate(elem)
}
