use core::prelude::*;
use core::fmt;
use core::fmt::{Debug, Formatter};
use mutex::Mutex;
use util::{PAGE_SIZE, is_page_aligned};
use util::rawbox::{RawBox, Unallocated};
logger_init!(Trace);

static FREE_FRAME_LIST: Mutex<Option<RawBox<Frame>>> = static_mutex!(None);

/// A frame available for allocation.
pub struct Frame {
    next: Option<RawBox<Frame>>
}

impl Frame {

    /// Creates and initializes a free frame from a unique memory address.
    ///
    /// # Safety
    ///
    /// This is unsafe because the caller must guarantee that address is actually unique.
    pub unsafe fn from_addr(addr: usize) -> RawBox<Frame> {
        assert!(is_page_aligned(addr));
        let mut frame = RawBox::from_raw(addr as *mut Frame);
        frame.next = None;
        frame
    }

}

impl Unallocated for Frame {
    fn get_free_size(&self) -> usize { PAGE_SIZE }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Frame(0x{:x})", self as *const Frame as usize)
    }
}

/// Adds a range of physical memory to the free frame list. This assumes that these ranges do not
/// overlap any ranges already added to the free frame list.
pub fn add_range(start: usize, end: usize) {
    assert!(is_page_aligned(start));
    assert!(is_page_aligned(end));
    trace!("adding range: {:x}-{:x}", start, end);

    let mut head = FREE_FRAME_LIST.lock();
    for frame_addr in (start..end).step_by(PAGE_SIZE) {
        // Filter out the zero frame because a frame with address 0 is "not present". This
        // check probably does not belong here.
        if frame_addr == 0 { continue }
        
        // Add the frame to the free frame list. We know this is safe because we assume that this
        // range does not overlap any ranges already added to the free frame list.
        let mut frame = unsafe { Frame::from_addr(frame_addr) };
        frame.next = head.take();
        *head = Some(frame);
    }
}

/// Tries to allocate a free frame. 
///
/// # Failures
///
/// Returns `None` if the free frame list is empty.
pub fn get_frame() -> Option<RawBox<Frame>> {
    let mut head = FREE_FRAME_LIST.lock();
    head.take().and_then(|mut frame| {
        assert!(is_page_aligned(&*frame as *const Frame as usize));
        *head = frame.next.take();
        Some(frame)
    })
}

/// Returns a frame to the free frame list.
pub fn return_frame(mut frame: RawBox<Frame>) {
    assert!(is_page_aligned(&*frame as *const Frame as usize));
    let mut head = FREE_FRAME_LIST.lock();
    frame.next = head.take();
    *head = Some(frame);
}

pub fn init () {

}


