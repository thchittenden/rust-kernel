use collections::vec::Vec;
use alloc::boxed::Box;
logger_init!(Trace);

#[inline(never)]
pub fn test() {
    trace!("\ntesting vec");
    let mut x = Vec::new(4).unwrap();

    for i in 10 .. 20 {
        trace!("pushing {}", i);
        assert!(x.push(i).is_ok());
    }
    
    for i in 19 .. 9 {
        assert!(x.pop().unwrap() == i);
    }

    // Test drops
    let mut y = Vec::new(4).unwrap();
    y.push(Box::new(3).unwrap()).unwrap();
    y.push(Box::new(4).unwrap()).unwrap();
    y.push(Box::new(5).unwrap()).unwrap();

    let new = y.split_at(1).unwrap();
    assert!(y.len() == 1);
    assert!(new.len() == 2);
}

