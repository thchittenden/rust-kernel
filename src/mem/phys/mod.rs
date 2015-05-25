use core::prelude::*;
use sync::mutex::Mutex;
use util;
use console;
logger_init!(Trace);

static FREE_FRAME_LIST: Mutex<Option<Frame>> = static_mutex!(None);

struct Frame {
    next: Option<&'static Frame>
}

pub fn init() {

}

pub fn add_range(base: usize, len: usize) {
    //assert!(util::is_page_aligned(base));
    //assert!(util::is_page_aligned(len));
    trace!("adding range: {:x} - {:x}", base, base + len);
}

