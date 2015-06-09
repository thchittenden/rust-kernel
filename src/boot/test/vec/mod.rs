use collections::vec::Vec;
logger_init!(Trace);

pub fn test() {
    trace!("\ntesting vec");
    let mut x = Vec::new(4).unwrap();

    for i in 10 .. 20 {
        trace!("pushing {}", i);
        let suc = x.push(i);
        assert!(suc);
    }
    
    for i in 19 .. 9 {
        assert!(x.pop().unwrap() == i);
    }
}

