#![crate_name="alloc"]
#![crate_type="rlib"]
#![feature(no_std,lang_items,unique,core,filling_drop)]
#![no_std]
//!
//! The kernel allocation library.
//!
//! This module is responsible for performing all heap allocations for the kernel. It serves as a
//! front-end for various back-end allocators. These back-end allocators merely need to implement
//! the Allocator trait.
//!
//! This module additionally contains a definition of an owned box similar to the one in Rust's
//! standard library. 
//!
//! The major ways in in which this module differs from the standard library's allocation library
//! is that it handles allocation failure gracefully where the standard library aborts. Since
//! aborting the kernel on an allocation failure is unaccceptable, all allocation procedures now
//! return an Option. This includes `Box::new`.
//!

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate mutex;

pub mod boxed;
pub mod rc;
mod lmm;

use core::prelude::*;
use core::mem;
use core::mem::min_align_of;
use core::ptr::Unique;
use mutex::Mutex;
use lmm::{LMMAllocator, LMM_ALLOCATOR_INIT};

/// An interface for dealing with Allocator back-ends. Implementors only need implement
/// `allocate_raw` and `deallocate_raw`.
///
/// This should likely be extended with initialization procedures as well as methods for
/// adding/removing regions from the heap.
trait Allocator {

    /// Tries to allocate `size` bytes aligned to `align` on the heap. Returns the address of the
    /// allocation if successful and `None` otherwise.
    fn allocate_raw(&mut self, size: usize, align: usize) -> Option<usize>;

    /// Tries to reallocate the allocation at `addr` so that it can allocate `size` bytes. If more
    /// appropriate slot cannot be found, returns the original allocation.
    fn reallocate_raw(&mut self, old_addr: usize, old_size: usize, new_size: usize, align: usize) -> Result<usize, usize>;

    /// Frees `size` bytes of allocated memory located at `addr`. 
    fn deallocate_raw(&mut self, addr: usize, size: usize);

    /// Tries to allocate an object aligned to `align`. Returns a unique pointer to the object if
    /// successful and `None` otherwise.
    fn allocate_aligned<T>(&mut self, elem: T, align: usize) -> Option<Unique<T>> {
        let alloc = self.allocate_raw(mem::size_of::<T>(), align);
        alloc.map(|addr| {
            let uniq = unsafe { Unique::new(addr as *mut T) };
            unsafe { **uniq = elem }
            uniq
        })
    }
  
    /// Tries to allocate an object from a constructor. Returns a unique pointer to the object if
    /// successful and `None` otherwise.
    fn allocate_emplace<F, T>(&mut self, init: F) -> Option<Unique<T>> where F: Fn(&mut T) {
        let alloc = self.allocate_raw(mem::size_of::<T>(), min_align_of::<T>());
        alloc.map(|addr| {
            let uniq = unsafe { Unique::new(addr as *mut T) };
            init(unsafe { &mut**uniq  });
            uniq
        })
    }
   
    /// Tries to allocate an object at its default alignment. Returns a unique pointer to the
    /// object if successful and `None` otherwise.
    fn allocate<T>(&mut self, elem: T) -> Option<Unique<T>> {
        self.allocate_aligned(elem, min_align_of::<T>())
    }

    /// Frees an object located on the heap.
    fn deallocate<T: ?Sized>(&mut self, mut elem: Unique<T>) {
        let addr = unsafe { elem.get_mut() } as *const _ as *const () as usize;
        let size = mem::size_of_val(unsafe { elem.get() });
        self.deallocate_raw(addr, size);
    }

}

static ALLOCATOR: Mutex<LMMAllocator> = static_mutex!(LMM_ALLOCATOR_INIT);

/// Initializes the allocation library and allocates all memory between `__heap_start` and
/// `__heap_end` to the allocator.
pub fn init() {
    let heap_start = linker_sym!(__heap_start);
    let heap_end = linker_sym!(__heap_end);
    ALLOCATOR.lock().init(heap_start, heap_end);
}

/// Tries to allocate space on the heap and returns a unique pointer to it.
///
/// # Failures
///
/// Fails if the heap cannot find a slot big enough to accomodate the requested object.
pub extern fn allocate_raw(size: usize, align: usize) -> Option<usize> {
    ALLOCATOR.lock().allocate_raw(size, align)
}

/// Tries to reallocate a space on the heap to accomodate a new size. Returns Ok(addr) if succesful
/// or Err(old_addr) if unsuccesful.
pub extern fn reallocate_raw(old_addr: usize, old_size: usize, new_size: usize, align: usize) -> Result<usize, usize> {
    ALLOCATOR.lock().reallocate_raw(old_addr, old_size, new_size, align)
}   

/// Deallocates a space on the heap. 
pub extern fn deallocate_raw(addr: usize, size: usize) {
    ALLOCATOR.lock().deallocate_raw(addr, size)
}

/// Tries to allocate an object to the heap and returns a unique pointer to it.
///
/// # Failures
///
/// Fails if the heap cannot find a slot big enough to accomodate the requested object.
pub extern fn allocate<T>(elem: T) -> Option<Unique<T>> {
    ALLOCATOR.lock().allocate(elem)
}

/// Tries to allocate an object on the heap from the given constructor and returns a unique pointer
/// to it.
///
/// # Failures
///
/// Fails if the heap cannot find a slot big enough to accomodate the requested object.
pub extern fn allocate_emplace<F, T>(init: F) -> Option<Unique<T>> where F: Fn(&mut T) {
    ALLOCATOR.lock().allocate_emplace(init)
}

/// Frees an object on the heap. If this object implements Drop, its destructor WILL NOT BE CALLED.
/// This is up to the caller of deallocate to perform. TODO This may want to be changed.
pub extern fn deallocate<T: ?Sized>(elem: Unique<T>) {
    ALLOCATOR.lock().deallocate(elem)
}
