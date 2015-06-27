use core::prelude::*;
use core::str;
use collections::string::String;

pub const PATH_SEP: &'static str = "/";

#[derive(Debug)]
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

    pub fn dirs(&self) -> str::Split<&'static str> {
        self.path.as_str().split(PATH_SEP)
    }

}

