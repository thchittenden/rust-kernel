use core::prelude::*;
use core::mem;
use core::fmt;
use core::fmt::{Debug, Formatter};
use sync::mutex::Mutex;
use util::{PAGE_SIZE, is_page_aligned};
use ::rawbox::RawBox;
logger_init!(Trace);

static FREE_FRAME_LIST: Mutex<Option<RawBox<Frame>>> = static_mutex!(None);

pub struct Frame {
    next: Option<RawBox<Frame>>
}

impl Frame {
    // Creates and initializes a free frame from a memory address.
    pub fn from_addr(addr: usize) -> RawBox<Frame> {
        assert!(is_page_aligned(addr));
        let mut frame = RawBox::from_raw(addr as *mut Frame);
        frame.next = None;
        frame
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Frame(0x{:x})", self as *const Frame as usize)
    }
}

impl RawBox<Frame> {
    // Convert a frame to an appropriate type.
    pub fn allocate<T>(self) -> RawBox<T> {
        assert!(mem::size_of::<T>() <= PAGE_SIZE);
        RawBox::from_raw(self.to_raw() as *mut T)
    }

    pub fn to_addr(self) -> usize {
        self.to_raw() as usize
    }
}

pub fn add_range(start: usize, end: usize) {
    assert!(is_page_aligned(start));
    assert!(is_page_aligned(end));
    trace!("adding range: {:x}-{:x}", start, end);

    let mut head = FREE_FRAME_LIST.lock().unwrap();
    for frame_addr in (start..end).step_by(PAGE_SIZE) {
        // Add the frame to the free frame list.
        let mut frame = Frame::from_addr(frame_addr);
        frame.next = head.take();
        *head = Some(frame);
    }
}

pub fn get_frame() -> Option<RawBox<Frame>> {
    let mut head = FREE_FRAME_LIST.lock().unwrap();
    head.take().and_then(|mut frame| {
        assert!(is_page_aligned(&*frame as *const Frame as usize));
        *head = frame.next.take();
        Some(frame)
    })
}

pub fn return_frame(mut frame: RawBox<Frame>) {
    assert!(is_page_aligned(&*frame as *const Frame as usize));
    let mut head = FREE_FRAME_LIST.lock().unwrap();
    frame.next = head.take();
    *head = Some(frame);
}

pub fn init () {

}

