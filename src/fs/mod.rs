#![crate_name="fs"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,core_str_ext,const_fn)]
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
use util::global::Global;
use collections::string::String;
use vfs::VFS;

const PATH_SEP: &'static str = "/";


pub trait File {
    
    unsafe fn read(&self, into: usize, offset: usize, count: usize) -> usize;

    unsafe fn write(&mut self, from: usize, offset: usize, count: usize) -> usize;

}

pub type FileIter<'a> = Iterator<Item=&'a String>;

pub trait FileCursor {
    
    fn open(&self, file: String) -> Option<Box<File>>;

    fn list<'a>(&'a self) -> Option<Box<Iterator<Item=&'a String> + 'a>>;

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

static ROOT: Global<VFS> = Global::new();

pub fn init() {
    let root = VFS::new().unwrap();
    let cursor = root.create_cursor().unwrap();
    cursor.mkdir("dev").unwrap();
}
