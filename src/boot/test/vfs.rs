use core::prelude::*;
use core::atomic::AtomicUsize;
use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use fs::*;
use collections::string::String;
use io::keyboard;
use io::console::CON;
logger_init!(Trace);

struct S {
    rc: AtomicUsize,
    foo: usize,
}

struct Y {
    rc: AtomicUsize,
    s: &'static str,
}

impl S {
    fn new(foo: usize) -> S {
        S {
            rc: AtomicUsize::new(0),
            foo: foo,
        }
    }
}

impl Y {
    fn new(s: &'static str) -> Y {
        Y { 
            rc: AtomicUsize::new(0),
            s: s,
        }
    }
}

impl HasRc for S {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

impl HasRc for Y {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

#[inline(never)]
pub fn test() {

    let mut cursor = root_cursor();
    let count = cursor.count();
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
    assert!(cursor.remove_node("test1").is_err());
    cursor.cd(Path::new(String::from_str("test1"))).unwrap();
    assert!(cursor.remove_node("test2").is_ok());
    cursor.cd(Path::new(String::from_str(".."))).unwrap();
    assert!(cursor.remove_node("test1").is_ok());

    // Test deep.
    for i in 0..16 {
        if i == 9 {
            trace!("break!");
        }
        let mut name = String::new();
        print!(name, "test{}", i);
        cursor.make_node(name.clone().unwrap()).unwrap();
        cursor.cd(Path::new(name)).unwrap();
        trace!("curdir: {}", cursor.get_cd());
    }
    // Test back.
    for i in 0..16 {
        let j = 15 - i;
        let mut name = String::new();
        print!(name, "test{}", j);
        cursor.cd(Path::new(String::from_str(".."))).unwrap();
        cursor.remove_node(name.as_str()).unwrap();
        trace!("curdir: {}", cursor.get_cd());
    }

    // Test again. Scoped so we drop the reader lock.
    {
        assert!(cursor.count() == count);
        let mut iter = cursor.list().unwrap();
        while let Some(s) = iter.next() {
            trace!("dir: {}", s);
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

    trace!("testing objects");
    let s1 = Rc::new(Box::new(S::new(4)).unwrap());
    let s2 = Rc::new(Box::new(S::new(3)).unwrap());
    let y1 = Rc::new(Box::new(Y::new("blah")).unwrap());
    let y2 = Rc::new(Box::new(Y::new("barz")).unwrap());
    cursor.make_object(String::from_str("obj1"), s1).unwrap();
    cursor.make_object(String::from_str("obj2"), s2).unwrap();
    cursor.make_object(String::from_str("obj3"), y1).unwrap();
    cursor.make_object(String::from_str("obj4"), y2).unwrap();

    let rs1 = cursor.open_object::<S>("obj1").unwrap();
    let rs2 = cursor.open_object::<S>("obj2").unwrap();
    let ry1 = cursor.open_object::<Y>("obj3").unwrap();
    let ry2 = cursor.open_object::<Y>("obj4").unwrap();

    assert!(rs1.foo == 4);
    assert!(rs2.foo == 3);
    assert!(ry1.s == "blah");
    assert!(ry2.s == "barz");
    assert!(cursor.open_object::<S>("obj3").is_err());
    assert!(cursor.open_object::<Y>("obj1").is_err());

    cursor.remove_object("obj1").unwrap();
    cursor.remove_object("obj2").unwrap();
    cursor.remove_object("obj3").unwrap();
    cursor.remove_object("obj4").unwrap();
}

fn is_whitespace(c: char) -> bool {
    match c {
        ' ' => true,
        '\t' => true,
        '\n' => true,
        '\r' => true,
        _ => false
    }
}

pub fn vfs_shell() -> ! {

    let mut cursor = root_cursor();
    println!(CON, "");
    loop {
        print!(CON, "{} > ", cursor.get_cd());
        let cmd_full = keyboard::getline().unwrap();
        let cmd_trim = cmd_full.as_str().trim_matches(is_whitespace);
        let mut words = cmd_trim.split(is_whitespace);
        let cmd = words.next().unwrap_or("{empty}");
        println!(CON, "{}", cmd_trim);
        match cmd {
            "ls" => {
                debug!("ls");
                let mut iter = cursor.list().unwrap();
                while let Some(file) = iter.next() {
                    println!(CON, "{}", file);
                }
            }
            "cd" => { 
                debug!("cd");
                match words.next() {
                    None => println!(CON, "cd PATH"),
                    Some(arg) => {
                        let mut path_str = String::new();
                        path_str.append(arg).unwrap();
                        match cursor.cd(Path::new(path_str)) {
                            Err(b) => println!(CON, "error: {:?}", b),
                            Ok(()) => { }
                        }
                    }
                }
            }
            "mkdir" => { 
                debug!("mkdir") ;
                match words.next() {
                    None => println!(CON, "mkdir NAME"),
                    Some(arg) => {
                        let mut name = String::new();
                        name.append(arg).unwrap();
                        match cursor.make_node(name) {
                            Err(b) => println!(CON, "error: {:?}", b),
                            Ok(()) => { }
                        }
                    }
                }
            }
            "rmdir" => {
                debug!("rmdir");
                match words.next() {
                    None => println!(CON, "rmdir NAME"),
                    Some(arg) => {
                        match cursor.remove_node(arg) {
                            Err(b) => println!(CON, "error: {:?}" ,b),
                            Ok(()) => { }
                        }
                    }
                }
            }
            _ => {
                println!(CON, "unknown command '{}'", cmd);
            }
        }
    }

}
