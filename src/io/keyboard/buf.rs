use core::prelude::*;
use core::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

const BUF_SIZE: usize = 256;

pub struct KeyboardBuffer {
    buf: [char; BUF_SIZE],
    head: AtomicUsize, // dequeue here
    tail: AtomicUsize, // enqueue here
}
pub const KEYBOARD_BUFFER_INIT: KeyboardBuffer = KeyboardBuffer {
    buf: ['\0'; BUF_SIZE],
    head: ATOMIC_USIZE_INIT,
    tail: ATOMIC_USIZE_INIT,
};

impl KeyboardBuffer {
    
    fn get_head(&self) -> usize {
        self.head.load(Ordering::Relaxed)
    }

    fn set_head(&self, head: usize) {
        self.head.store(head, Ordering::Relaxed);
    }

    fn get_tail(&self) -> usize {
        self.tail.load(Ordering::Relaxed)
    }

    fn set_tail(&self, tail: usize) {
        self.tail.store(tail, Ordering::Relaxed);
    }
    
    pub fn is_full(&self) -> bool {
        self.get_head() == (self.get_tail() + 1) % BUF_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.get_head() == self.get_tail()
    }

    pub fn enqueue(&mut self, c: char) {
        if !self.is_full() {
            self.buf[self.get_tail()] = c;
            self.set_tail((self.get_tail() + 1) % BUF_SIZE);
        }
    }

    pub fn dequeue(&mut self) -> Option<char> {
        if !self.is_empty() {
            let c = self.buf[self.get_head()];
            self.set_head((self.get_head() + 1) % BUF_SIZE);
            Some(c)
        } else {
            None
        }
    }

}
