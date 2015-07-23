use core::{fmt, mem};
use core::intrinsics::drop_in_place;
use core::ops::{Deref, DerefMut};
use util::{asm, is_page_aligned, PAGE_SIZE};

pub struct Page<T> {
    ptr: *mut T
}

impl<T> Page<T> {
    
    /// Creates and initializes a page from a unique memory address.
    ///
    /// # Safety
    ///
    /// This is unsafe because the caller must guarantee that address is actually unique and allows
    /// constructing arbitrary types from possibly uninitialized memory.
    pub unsafe fn from_addr(addr: usize) -> Page<T> {
        assert!(is_page_aligned(addr));
        Page { ptr: addr as *mut T }
    }

    pub fn allocate<U>(self, val: U) -> Page<U> {
        assert!(mem::size_of::<U>() <= PAGE_SIZE);

        // Extract the frame address and drop the old value.
        let addr = self.ptr as usize;
        unsafe { drop_in_place(self.ptr) };
        mem::forget(self);

        // Create the new frame and populate with the new value.
        let mut page = Page { ptr: addr as *mut U };
        *page = val;
        page
    }

    
}

impl<T> Deref for Page<T> {
    type Target = T;
    fn deref(&self) -> &T {
        assert!(asm::paging_enabled());
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Page<T> {
    fn deref_mut(&mut self) -> &mut T {
        assert!(asm::paging_enabled());
        unsafe { &mut *self.ptr }
    }
}

impl<T> fmt::Debug for Page<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Page(0x{:x})", self.ptr as usize)
    }
}

