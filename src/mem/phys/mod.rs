use core::prelude::*;
use sync::mutex::Mutex;
use util;

static FREE_FRAME_LIST: Mutex<Option<Frame>> = static_mutex!(None);

struct Frame {
    next: Option<&'static Frame>
}

pub fn init() {

}

pub fn add_range(base: usize, len: usize) {
    assert!(util::is_page_aligned(base));
    assert!(util::is_page_aligned(len));


}
