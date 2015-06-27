use core::prelude::*;
use fs::*;
logger_init!(Trace);

pub fn test() {

    let cursor = root_cursor();
    trace!("ls /");
    let mut iter = cursor.list().unwrap();
    while let Some(s) = iter.next() {
        trace!("  {}", s);
    }

}
