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
use util::global::Global;
use collections::string::String;
use vfs::VFS;
use path::Path;
use util::KernResult;
logger_init!(Trace);

pub const PATH_SEP: &'static str = "/";
pub const PARENT_DIR: &'static str = "..";

pub trait Node : HasRc {
    
    fn count(&self) -> usize;

    fn list<'a>(&'a self) -> KernResult<Box<Iterator<Item=&'a str> + 'a>>;

    fn make_file(&self, file: String) -> KernResult<()>;

    fn make_node(&self, node: String) -> KernResult<()>; 
    
    fn open_file(&self, file: &str) -> KernResult<Box<File>>;

    fn open_node(&self, node: &str) -> KernResult<Rc<Node>>;

    fn remove_file(&self, file: &str) -> KernResult<()>;

    fn remove_node(&self, node: &str) -> KernResult<()>;

    fn mount(&self, node: String, fs: Box<FileSystem>) -> KernResult<()>;

    fn unlink(&self) -> KernResult<()>;

}

pub trait FileSystem {
    
    fn root_node(&self) -> KernResult<Rc<Node>>;

    fn set_parent(&mut self, parent: Option<Rc<Node>>);

}

pub trait File {
    
    unsafe fn read(&self, into: usize, offset: usize, count: usize) -> usize;

    unsafe fn write(&mut self, from: usize, offset: usize, count: usize) -> usize;

}

pub struct FileCursor {
    curdir: Path, // Probably want to make this a stack of nodes.
    node: Rc<Node>,
}

impl FileCursor {
   
    pub fn count(&self) -> usize {
        trace!("count {}", self.curdir);
        self.node.count()
    }

    pub fn list<'a>(&'a self) -> KernResult<Box<Iterator<Item=&'a str> + 'a>> {
        trace!("listing {}", self.curdir);
        self.node.list()
    }

    pub fn make_node(&self, node: String) -> KernResult<()> {
        trace!("making node {} at {}", node, self.curdir);
        self.node.make_node(node)
    }

    pub fn remove_node(&self, name: &str) -> KernResult<()> {
        trace!("removing node {} at {}", name, self.curdir);
        self.node.remove_node(name)
    }

    pub fn cd(&mut self, path: Path) -> KernResult<()> {
        trace!("trying to cd from {} to {}/{}", self.curdir, self.curdir, path);
        let mut cur = self.node.clone();
        for dir in path.dirs() {
            cur = try!(cur.open_node(dir));
            if dir == PARENT_DIR {
                // This could be optimized so it can't fail.
                try!(self.curdir.pop_dir());
            } else {
                try!(self.curdir.push_dir(dir));
            }
            
        }
        self.node = cur;
        Ok(())
    }

    pub fn get_cd(&self) -> &Path {
        &self.curdir
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
    cursor.make_node(String::from_str("dev")).unwrap();
}

