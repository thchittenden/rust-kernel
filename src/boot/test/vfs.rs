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

    cursor.make_node(String::from_str("test1")).unwrap();
    cursor.cd(Path::new(String::from_str("test1"))).unwrap();
    cursor.make_node(String::from_str("test2")).unwrap();
    cursor.cd(Path::new(String::from_str("test2"))).unwrap();
    trace!("curdir: {}", cursor.get_cd());
    cursor.cd(Path::new(String::from_str(".."))).unwrap();
    trace!("cd .. => curdir: {}", cursor.get_cd());
    cursor.cd(Path::new(String::from_str(".."))).unwrap();
    trace!("cd .. => curdir: {}", cursor.get_cd());
    {
        trace!("ls curdir");
        let mut iter = cursor.list().unwrap();
        while let Some(s) = iter.next() {
            trace!("  {}", s);
        }
    }


    let mut path = Path::new(String::from_str("/"));
    path.push_dir("a").unwrap();
    path.push_dir("deep").unwrap();
    path.push_dir("path").unwrap();
    path.push_dir("here").unwrap();
    trace!("path: {}", path);
    while let Ok(Some(dir)) = path.pop_dir() {
        trace!("path: {} + {}", path, dir);
    }


}

