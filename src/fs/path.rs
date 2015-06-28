use core::prelude::*;
use core::{str, mem, fmt};
use collections::string::String;
use util::KernResult;

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

    pub fn append(&mut self, other: Path) -> KernResult<()> {
        if other.is_absolute() {
            self.path = other.path;
        } else {
            if self.is_root() {
                try!(self.path.append(other.path.as_str()));
            } else {
                // ONLY the root directory has a trailing '/' so add one otherwise..
                // TODO transactional.
                try!(self.path.append(PATH_SEP));
                try!(self.path.append(other.path.as_str()));
            }
        }
        Ok(())
    }

    pub fn push_dir(&mut self, dir: &str) -> KernResult<()> {
        if self.is_root() {
            try!(self.path.append(dir));
        } else {
            try!(self.path.append(PATH_SEP));
            try!(self.path.append(dir));
        }
        Ok(())
    }

    pub fn pop_dir(&mut self) -> KernResult<Option<String>> {
        let idx = self.path.as_str().rfind(PATH_SEP);
        match idx {
            None => {
                // No separators in the string, that means its just a single directory
                let mut res = String::new();
                mem::swap(&mut self.path, &mut res);
                Ok(Some(res))
            }
            Some(idx) => {
                if self.is_root() {
                    Ok(None)
                } else {
                    let new = try!(self.path.split_at(idx + 1));
                    if idx != 0 {
                        // Remove the trailing slash if we're not at the root.
                        self.path.pop();
                    }
                    Ok(Some(new))
                }
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
