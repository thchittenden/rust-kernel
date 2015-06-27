use core::prelude::*;
use core::str;
use collections::string::String;

pub const PATH_SEP: &'static str = "/";

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

