use core::prelude::*;
use core::{fmt, mem};
use core::atomic::{AtomicUsize, Ordering};
use core::fmt::{Debug, Formatter};
use core::intrinsics::drop_in_place;
use core::ops::{Deref, DerefMut};
use mutex::Mutex;
use util::{PAGE_SIZE, is_page_aligned};
use util::rawbox::RawBox;
use util::KernResult;
use util::KernError::*;
logger_init!(Trace);

static FREE_FRAME_LIST: Mutex<FreeFrameList> = Mutex::new(FreeFrameList::new());

struct FreeFrame {
    next: Option<RawBox<FreeFrame>>
}

impl FreeFrame {
    
    /// Constructs a free frame from an address.
    ///
    /// # Safety
    ///
    /// This is unsafe because the caller must ensure the address is unique.
    unsafe fn from_addr(addr: usize) -> RawBox<FreeFrame> {
        assert!(is_page_aligned(addr));
        let mut frame = RawBox::from_raw(addr as *mut FreeFrame);
        frame.next = None;
        frame
    }

}

/// A frame available for allocation.
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

    pub fn allocate<U>(self, val: U) -> Frame<U> {
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
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Frame<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for Frame<T> {
    fn drop(&mut self) {
        unsafe { drop_in_place(self.ptr) };
        let frame = unsafe { FreeFrame::from_addr(self.ptr as usize) };
        FREE_FRAME_LIST.lock().return_frame(frame);
    }
}



impl<T> Debug for Frame<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Frame(0x{:x})", self.ptr as usize)
    }
}

/// The free frame list.
struct FreeFrameList {
    head: Option<RawBox<FreeFrame>>,
    max: usize,
    count: usize,
    reserved: usize,
}

impl FreeFrameList {
    const fn new() -> FreeFrameList {
        FreeFrameList {
            head: None,
            max: 0,
            count: 0,
            reserved: 0,
        }
    }

    fn initialize(&mut self, lo: usize, hi: usize) {
        assert!(is_page_aligned(lo));
        assert!(is_page_aligned(hi));
        for addr in (lo..hi).step_by(PAGE_SIZE) {
            // Filter out the zero frame because a frame with address 0 is "not present". This
            // check probably does not belong here.
            if addr == 0 { continue }
            
            // Construct the frame and adds it to the free list.
            let mut frame = unsafe { FreeFrame::from_addr(addr) };
            frame.next = self.head.take();
            self.head = Some(frame);
            self.max += 1;
            self.count += 1;
        }
    }

    fn reserve(&mut self, count: usize) -> KernResult<()> {
        if self.reserved + count <= self.count {
            self.reserved += count;
            Ok(())
        } else {
            Err(OutOfMemory)
        }
    }

    fn unreserve(&mut self, count: usize) {
        assert!(self.reserved >= count);
        self.reserved -= count;
    }

    fn get_unreserved(&mut self) -> KernResult<Frame<()>> {
        if self.count > self.reserved {
            let mut top = self.head.take().unwrap();
            let next = top.next.take();
            self.head = next;
            self.count -= 1;
            Ok(unsafe { Frame::from_addr(top.into_raw() as usize) })
        } else {
            Err(OutOfMemory)
        }
    }

    fn get_reserved(&mut self) -> Frame<()> {
        assert!(self.reserved > 0);
        let mut top = self.head.take().unwrap();
        let next = top.next.take();
        self.head = next;
        self.count -= 1;
        self.reserved -= 1;
        unsafe { Frame::from_addr(top.into_raw() as usize) }
    }

    fn return_frame(&mut self, mut frame: RawBox<FreeFrame>) {
        frame.next = self.head.take();
        self.head = Some(frame);
        self.count += 1;
    }
}

/// A struct representing how many frames an address space has reserved. It must always be possible
/// to retrieve this many frames from the free frame list.
pub struct FrameReserve {
    count: AtomicUsize,
}

impl FrameReserve {
    
    pub fn new() -> FrameReserve {
        FrameReserve { count: AtomicUsize::new(0) }
    }

    pub fn reserve(&self, count: usize) -> KernResult<()> {
        try!(FREE_FRAME_LIST.lock().reserve(count));
        self.count.fetch_add(count, Ordering::Relaxed);
        Ok(())
    }

    pub fn unreserve(&self, count: usize) {
        FREE_FRAME_LIST.lock().unreserve(count)
    }

    pub fn split(&self, new_count: usize) -> FrameReserve {
        self.count.fetch_sub(new_count, Ordering::Relaxed);
        FrameReserve { count: AtomicUsize::new(new_count) }
    }

    pub fn get_frame(&self) -> Frame<()> {
        assert!(self.count.fetch_sub(1, Ordering::Relaxed) > 0);
        FREE_FRAME_LIST.lock().get_reserved()
    }

    pub fn get_frame_unreserved(&self) -> KernResult<Frame<()>> {
        FREE_FRAME_LIST.lock().get_unreserved()
    }
}

impl Drop for FrameReserve {
    fn drop(&mut self) {
        let count = self.count.swap(0, Ordering::Relaxed);
        FREE_FRAME_LIST.lock().unreserve(count);
    }   
}

/// Adds a range of physical memory to the free frame list. This assumes that these ranges do not
/// overlap any ranges already added to the free frame list.
pub fn add_range(start: usize, end: usize) {
    trace!("adding range: {:x}-{:x}", start, end);
    FREE_FRAME_LIST.lock().initialize(start, end);
}

pub fn init () {

}


