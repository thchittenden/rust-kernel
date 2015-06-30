use core::prelude::*;
use fs::*;
use fs::path::Path;
use collections::string::String;
use io::keyboard;
use io::console::CON;
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

    // Test again.
    assert!(cursor.count() == 1);
    let mut iter = cursor.list().unwrap();
    while let Some(s) = iter.next() {
        trace!("dir: {}", s);
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
