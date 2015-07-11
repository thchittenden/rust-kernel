use core::prelude::*;
use alloc::boxed::Box;
logger_init!(Trace);


trait Foo {
    fn grok(&self) -> usize;
}

struct Bar { a: usize, b: isize }

impl Foo for Bar {
    fn grok(&self) -> usize { self.a + self.b as usize }
}

fn test_unsized(a: Box<Foo>) {
    drop(a)
}

struct Nested {
    x: Box<usize>
}

#[inline(never)]
pub fn test() {

    trace!("\ntesting boxes");
    // Test recursive drops.
    let x = Box::new(3).unwrap();
    let y = Box::new(x).unwrap();
    trace!("got {}", y);
    trace!(" or {}", **y);
    assert!(**y == 3);
    drop(y);

    // Test unsized drops.
    let a = Bar { a: 1, b: 2 };
    let b = Box::new(a).unwrap();
    test_unsized(b);

    let n = Nested { x: Box::new(3).unwrap() };
    assert!(*n.x == 3);
    drop(n);
}

