use core::prelude::*;
use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
use core::fmt;
logger_init!(Trace);

struct Baz {
    rc: AtomicUsize,
    val: usize
}

impl HasRc for Baz {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

impl fmt::Debug for Baz {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Baz {{ val: {:?} }}", self.val)
    }
}

#[inline(never)]
pub fn test() {

    trace!("\ntesting rc");
    let x = Box::new(Baz { rc: ATOMIC_USIZE_INIT, val: 4 }).unwrap(); 
    let rcx1 = Rc::new(x);
    let rcx2 = rcx1.clone();

    trace!("rcx1: {:?}", rcx1);
    trace!("rcx2: {:?}", rcx2);

    drop(rcx1);

    trace!("rcx2 still live: {:?}", rcx2);

    drop(rcx2);
}

