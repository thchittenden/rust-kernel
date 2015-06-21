#![crate_name="fs"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate alloc;
extern crate collections;
extern crate sync;

pub mod vfs;

use alloc::boxed::Box;
use core::prelude::*;
use core::str;
use collections::vec::Vec;
use util::global::Global;
use collections::string::String;
use vfs::VFS;

const PATH_SEP: &'static str = "/";


pub trait File {
    
    fn read(&mut self, addr: usize, bytes: usize) -> bool;

    fn write(&mut self, addr: usize, bytes: usize) -> bool;

    fn seek(&mut self, idx: usize) -> bool;

}

pub trait FileCursor {
    
    fn open(&self, path: Path) -> Option<Box<File>>;

    fn list(&self) -> Option<Vec<&String>>;

    fn mkdir(&self, name: &str) -> Option<Box<FileCursor>>;

    fn cd(&mut self, path: Path) -> bool;

}

pub trait FileSystem {

    fn create_cursor(&self) -> Option<Box<FileCursor>>;

}

pub struct Path {
    path: String
}

impl Path {

    pub fn new(s: String) -> Path {
        Path { path: s }
    }

    pub fn is_absolute(&self) -> bool {
        self.path.as_str().starts_with(PATH_SEP)
    }

}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a str;
    type IntoIter = str::Split<'a, &'static str>;
    fn into_iter(self) -> Self::IntoIter {
        self.path.as_str().split(PATH_SEP)
    }
}



static ROOT: Global<VFS> = global_init!();

pub fn init() {
    let root = VFS::new().unwrap();
    let cursor = root.create_cursor().unwrap();
    cursor.mkdir("dev").unwrap();
}
