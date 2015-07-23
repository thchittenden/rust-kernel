use core::prelude::*;
use core::intrinsics::drop_in_place;
use core::ops::{Deref, DerefMut};
use core::{mem, fmt};
use super::FREE_FRAME_LIST;
use util::{asm, is_page_aligned, PAGE_SIZE};
logger_init!(Trace);

/// An unallocated frame.
pub struct FreeFrame {
    pub next: Option<Frame<FreeFrame>>
}

impl FreeFrame {
    
    pub fn new() -> FreeFrame {
        FreeFrame { next: None }
    }

}

/// A pointer to a frame containing data of type T.
pub struct Frame<T> {
    ptr: *mut T
}

impl<T> Frame<T> {

    /// Creates and initializes a free frame from a unique memory address.
    ///
    /// # Safety
    ///
    /// This is unsafe because the caller must guarantee that address is actually unique and allows
    /// constructing arbitrary types from possibly uninitialized memory.
    pub unsafe fn from_addr(addr: usize) -> Frame<T> {
        assert!(is_page_aligned(addr));
        Frame { ptr: addr as *mut T }
    }

    /// Converts a frame into a unique memory address.
    ///
    /// # Safety
    ///
    /// This is unsafe because the caller must ensure this address is converted back into a Frame
    /// so that it's properly dropped.
    pub unsafe fn into_addr(self) -> usize {
        let addr = self.ptr as usize;
        mem::forget(self);
        addr
    }

    /// Gets the address of this frame.
    pub fn get_addr(&self) -> usize {
        self.ptr as usize
    }   

    /// Casts the frame to a different type. This is primarily useful in a paging environment
    /// because it will not dereference the frame in any way.
    ///
    /// # Safety
    ///
    /// This is unsafe because the old value will not be dropped and this may allow access to
    /// uninitialized memory.
    pub unsafe fn cast<U>(self) -> Frame<U> {
        assert!(mem::size_of::<U>() <= PAGE_SIZE);
        let addr = self.ptr as usize;
        mem::forget(self);
        Frame { ptr: addr as *mut U }
    }

    pub unsafe fn allocate_raw<U>(self) -> Frame<U> {
        assert!(!asm::paging_enabled());
        assert!(mem::size_of::<U>() <= PAGE_SIZE);
        
        // Extract the frame address and drop the old value.
        let addr = self.ptr as usize;
        drop_in_place(self.ptr);
        mem::forget(self);

        // Create the new frame.
        Frame { ptr: addr as *mut U }
    }

    pub fn unallocate(self) -> Frame<()> {
        self.allocate(())
    }

    pub fn allocate<U>(self, val: U) -> Frame<U> {
        assert!(!asm::paging_enabled());
        assert!(mem::size_of::<U>() <= PAGE_SIZE);

        // Extract the frame address and drop the old value.
        let addr = self.ptr as usize;
        unsafe { drop_in_place(self.ptr) };
        mem::forget(self);

        // Create the new frame and populate with the new value.
        let mut frame = Frame { ptr: addr as *mut U };
        *frame = val;
        frame
    }

    pub fn emplace<U, F>(self, init: F) -> Frame<U> where F: Fn(&mut U) {
        assert!(!asm::paging_enabled());
        assert!(mem::size_of::<U>() <= PAGE_SIZE);

        // Extract the frame address and drop the old value.
        let addr = self.ptr as usize;
        unsafe { drop_in_place(self.ptr) };
        mem::forget(self);
        
        let mut frame = Frame { ptr: addr as *mut U };
        init(frame.deref_mut());
        frame
    }
}

impl<T> Deref for Frame<T> {
    type Target = T;
    fn deref(&self) -> &T {
        assert!(!asm::paging_enabled());
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Frame<T> {
    fn deref_mut(&mut self) -> &mut T {
        assert!(!asm::paging_enabled());
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for Frame<T> {
    fn drop(&mut self) {
        trace!("dropping frame {:?}", self);
        // We know it is safe to construct a frame here because we're dropping the old frame.
        let frame = unsafe { Frame::<()>::from_addr(self.ptr as usize) };
        FREE_FRAME_LIST.lock().return_frame(frame);
    }
}

impl<T> fmt::Debug for Frame<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Frame(0x{:x})", self.ptr as usize)
    }
}


