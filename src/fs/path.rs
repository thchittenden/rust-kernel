use core::prelude::*;
use core::str;
use core::fmt;
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

    pub fn is_root(&self) -> bool {
        self.path.as_str() == "/"
    }

    pub fn append(&mut self, other: Path) -> bool {
        if other.is_absolute() {
            self.path = other.path;
            true
        } else {
            if self.is_root() {
                self.path.append(other.path.as_str())
            } else {
                // ONLY the root directory has a trailing '/' so add one otherwise..
                self.path.append(PATH_SEP) && self.path.append(other.path.as_str())
            }
        }
    }

    pub fn dirs(&self) -> str::Split<&'static str> {
        self.path.as_str().split(PATH_SEP)
    }

}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}
