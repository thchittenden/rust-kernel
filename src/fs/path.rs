use core::prelude::*;
use core::{str, mem, fmt};
use collections::string::String;
use util::KernResult;
use super::PATH_SEP;

#[derive(Debug)]
pub struct Path {
    path: String
}

impl Path {

    pub fn new(s: String) -> Path {
        Path { path: s }
    }

    pub fn clone(&self) -> KernResult<Path> {
        Ok(Path { path: try!(self.path.clone()) })
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
            if self.path.as_str().ends_with(PATH_SEP) {
                try!(self.path.append(other.path.as_str()));
            } else {
                let mut other_path = String::from_str(PATH_SEP);
                try!(other_path.append(other.path.as_str()));
                try!(self.path.append(other_path.as_str()));
            }
        }
        Ok(())
    }

    pub fn push_dir(&mut self, dir: &str) -> KernResult<()> {
        if self.path.as_str().ends_with(PATH_SEP) {
            try!(self.path.append(dir));
        } else {
            let mut other_path = String::from_str(PATH_SEP);
            try!(other_path.append(dir));
            try!(self.path.append(other_path.as_str()));
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
