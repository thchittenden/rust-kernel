use core::prelude::*;
use fs::*;
use fs::path::Path;
use collections::string::String;
logger_init!(Trace);

pub fn test() {

    let mut cursor = root_cursor();
    {
        trace!("ls /");
        let mut iter = cursor.list().unwrap();
        while let Some(s) = iter.next() {
            trace!("  {}", s);
        }
    }

    cursor.make_node(String::from_str("test1"));
    cursor.cd(Path::new(String::from_str("test1")));
    cursor.make_node(String::from_str("test2"));
    cursor.cd(Path::new(String::from_str("test2")));
    trace!("curdir: {:?}", cursor.get_cd());



}
