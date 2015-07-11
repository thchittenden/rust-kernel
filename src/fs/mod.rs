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
use alloc::rc::{Rc, HasRc, RcAny};
use core::prelude::*;
use core::any::Any;
use util::global::Global;
use collections::string::String;
use self::vfs::VFS;
use path::Path;
use util::KernResult;
use util::KernError::WrongType;
logger_init!(Trace);

pub const PATH_SEP: &'static str = "/";
pub const PARENT_DIR: &'static str = "..";

pub trait Node : HasRc {
    
    fn count(&self) -> usize;

    fn list<'a>(&'a self) -> KernResult<Box<Iterator<Item=&'a str> + 'a>>;

    fn make_file(&self, file: String) -> KernResult<()>;

    fn make_node(&self, node: String) -> KernResult<()>; 

    fn make_object(&self, name: String, obj: Box<RcAny>) -> KernResult<()>;
    
    fn open_file(&self, file: &str) -> KernResult<Box<File>>;

    fn open_node(&self, node: &str) -> KernResult<Rc<Node>>;

    fn open_object(&self, name: &str) -> KernResult<Rc<RcAny>>;

    fn remove_file(&self, file: &str) -> KernResult<()>;

    fn remove_node(&self, node: &str) -> KernResult<()>;

    fn remove_object(&self, name: &str) -> KernResult<Rc<RcAny>>;

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
    curdir: Path,
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

    pub fn make_object<T: Any + HasRc>(&self, name: String, obj: Box<T>) -> KernResult<()> {
        trace!("making object {} at {}", name, self.curdir);

        self.node.make_object(name, obj)
    }

    pub fn remove_object(&self, name: &str) -> KernResult<()> {
        trace!("removing object {} at {}", name, self.curdir);
        try!(self.node.remove_object(name));
        Ok(())
    }

    pub fn open_object<T: Any + HasRc>(&self, name: &str) -> KernResult<Rc<T>> {
        trace!("opening object {} at {}", name, self.curdir);
        let obj = try!(self.node.open_object(name));
        let any = obj.as_any();
        match any.downcast_ref::<T>() {
            None => Err(WrongType),
            Some(obj) => Ok(Rc::from_ref(&obj)),
        }
    }

    pub fn cd(&mut self, path: Path) -> KernResult<()> {
        trace!("trying to cd from {} to {}/{}", self.curdir, self.curdir, path);
        let mut cur = self.node.clone();
        
        // Get the cursor that points to the new directory.
        for dir in path.dirs() {
            cur = try!(cur.open_node(dir));
        }

        // If that succeeds, construct the path to the new cursor. TODO this sucks that we need to
        // allocate a new path in order to perform this transactionally. This "canonical" append
        // should be pushed down into Path (or CanonicalPath) so we can perform it transactionally.
        let mut new_path = try!(self.curdir.clone());
        for dir in path.dirs() {
            if dir == PARENT_DIR {
                try!(new_path.pop_dir());
            } else {
                try!(new_path.push_dir(dir));
            }
        }
         
        // Now update our state and return success.
        self.curdir = new_path;
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

