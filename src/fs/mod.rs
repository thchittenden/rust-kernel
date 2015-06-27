#![crate_name="fs"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,core_str_ext,const_fn)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;
extern crate sync;

pub mod path;
pub mod vfs;

use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::str;
use util::global::Global;
use collections::string::String;
use vfs::VFS;
use path::Path;
logger_init!(Trace);

pub trait Node : HasRc {
    
    fn list<'a>(&'a self) -> Option<Box<Iterator<Item=&'a String> + 'a>>;

    fn open_file(&self, file: String) -> Option<Box<File>>;

    fn open_node(&self, node: String) -> Option<Rc<Node>>;

    fn make_file(&self, file: String) -> bool;

    fn make_node(&self, node: String) -> bool; 

}

pub trait FileSystem {
    
    fn root_node(&self) -> Option<Rc<Node>>;

}

pub trait File {
    
    unsafe fn read(&self, into: usize, offset: usize, count: usize) -> usize;

    unsafe fn write(&mut self, from: usize, offset: usize, count: usize) -> usize;

}

pub struct FileCursor {
    curdir: Path,
    node: Rc<Node>,
}

impl FileCursor {
    
    pub fn list<'a>(&'a self) -> Option<Box<Iterator<Item=&'a String> + 'a>> {
        self.node.list()
    }

    pub fn make_node(&self, node: String) -> bool {
        self.node.make_node(node)
    }

}


static ROOT: Global<Rc<Node>> = Global::new();

pub fn root_cursor() -> FileCursor {
    FileCursor {
        curdir: Path::new(String::from_str("/")),
        node: ROOT.clone(),
    }
}

pub fn init() {
    debug!("initializing fs");
    let root = VFS::new().unwrap().root_node().unwrap();
    ROOT.init(root);

    // Populate a few initial directories.
    let cursor = root_cursor();
    cursor.make_node(String::from_str("dev"));
}

