use core::prelude::*;
use sync::mutex::Mutex;
use util::{PAGE_SIZE, is_page_aligned};
logger_init!(Trace);

static FREE_FRAME_LIST: Mutex<Option<*mut Frame>> = static_mutex!(None);

struct Frame {
    next: Option<*mut Frame>
}

impl Frame {
    fn from_addr(addr: usize) -> *mut Frame {
        // Creates and initializes a frame from a memory address.
        assert!(is_page_aligned(addr));
        let frame = addr as *mut Frame;
        unsafe { (*frame).next = None; };
        frame
    }
}

pub fn init() {
    
}

pub fn add_range(start: usize, end: usize) {
    assert!(is_page_aligned(start));
    assert!(is_page_aligned(end));
    trace!("adding range: {:x}-{:x}", start, end);

    let mut head = FREE_FRAME_LIST.lock().unwrap();
    for frame_addr in (start..end).step_by(PAGE_SIZE) {
        assert!(is_page_aligned(frame_addr));

        // Add the frame to the free frame list.
        let frame = Frame::from_addr(frame_addr);
        unsafe { (*frame).next = head.take(); };
        *head = Some(frame);
    }
}

pub fn get_frame() -> Option<usize> {
    let mut head = FREE_FRAME_LIST.lock().unwrap();
    head.and_then(|frame| {
        assert!(is_page_aligned(frame as usize));
        *head = unsafe { (*frame).next };
        Some(frame as usize)
    })
}

pub fn return_frame(frame_addr: usize) {
    assert!(is_page_aligned(frame_addr));
    let mut head = FREE_FRAME_LIST.lock().unwrap();
    let frame = Frame::from_addr(frame_addr);
    unsafe { (*frame).next = head.take(); };
    *head = Some(frame);
}
