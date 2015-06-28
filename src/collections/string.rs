use core::prelude::*;
use core::{fmt, str};
use core::hash::{Hash, Hasher};
use super::vec::Vec;
use util::KernResult;

/// A dynamically growable string. If the string is never modified there is no extra overhead. 
pub enum String {
    StaticString(&'static str),
    DynamicString(DynString)
}
use self::String::*;

impl String {

    /// Constructs a new empty string.
    pub fn new() -> String {
        StaticString("")
    }

    /// Constructs a new string from a static string.
    pub fn from_str(s: &'static str) -> String {
        StaticString(s)
    }

    fn is_static(&self) -> bool {
        match self {
            &StaticString(_) => true,
            &DynamicString(_) => false,
        }
    }

    fn make_dynamic(&mut self) -> KernResult<()> {
        *self = match self {
            &mut DynamicString(_) => return Ok(()),
            &mut StaticString(ref s) => { 
                let ds = try!(DynString::from_str(s));
                DynamicString(ds)
            }
        };
        Ok(())
    }

    pub fn append(&mut self, s: &str) -> KernResult<()> {
        if self.is_static() {
            try!(self.make_dynamic())
        } 
        match self {
            &mut StaticString(_) => unreachable!(),
            &mut DynamicString(ref mut ds) => ds.append(s)
        }
    }

    pub fn push(&mut self, c: char) -> KernResult<()> {
        if self.is_static() {
            try!(self.make_dynamic())
        } 
        match self {
            &mut StaticString(_) => unreachable!(),
            &mut DynamicString(ref mut ds) => ds.push(c)
        }
    } 

    pub fn pop(&mut self) -> Option<char> {
        match self {
            &mut StaticString(ref mut s) => {
                let len = s.len(); 
                if len == 0 {
                    None 
                } else {
                    let c = s.char_at_reverse(len);
                    *s = unsafe { s.slice_unchecked(0, len - c.len_utf8()) };
                    Some(c)
                }
            },
            &mut DynamicString(ref mut ds) => ds.pop()
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            &StaticString(ref s) => s,
            &DynamicString(ref ds) => ds.as_str()
        }
    }

    pub fn split_at(&mut self, idx: usize) -> KernResult<String> {
        try!(self.make_dynamic());
        match self {
            &mut StaticString(_) => unreachable!(),
            &mut DynamicString(ref mut ds) => {
               let new = try!(ds.arr.split_at(idx));
               Ok(DynamicString(DynString { arr: new }))
            }
        }
    }

}

impl PartialEq<String> for String {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for String { }

impl Hash for String {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_str().as_bytes())
    }
}

impl fmt::Write for String {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(try!(self.append(s)))
    }
}

impl fmt::Display for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Debug for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}


/// A dynamically growable string.
pub struct DynString {
    arr: Vec<u8>,
}

impl DynString {
    
    pub fn new() -> KernResult<DynString> {
        let vec = try!(Vec::new(16));
        Ok(DynString { arr: vec })
    }

    pub fn from_str(s: &str) -> KernResult<DynString> {
        let mut vec = try!(Vec::new(s.len()));
        for byte in s.as_bytes() {
            assert!(vec.push(*byte).is_ok())
        }
        Ok(DynString { arr: vec })
    }

    pub fn append(&mut self, s: &str) -> KernResult<()> {
        for byte in s.as_bytes() {
            try!(self.arr.push(*byte))
        }
        Ok(())
    }

    pub fn push(&mut self, c: char) -> KernResult<()> {
        try!(self.arr.reserve(c.len_utf8()));
        let last = self.arr.len();
        let slice = unsafe { self.arr.as_mut_slice_full() };
        assert!(c.encode_utf8(&mut slice[last..]).is_some());
        Ok(())
    }

    pub fn pop(&mut self) -> Option<char> {
        let len = self.arr.len();
        if len == 0 {
            None
        } else {
            let c = self.as_str().char_at_reverse(len);
            for _ in 0..c.len_utf8() {
                self.arr.pop();
            }
            Some(c)
        }
    }

    pub fn as_str(&self) -> &str {
        // We know this is safe because we only put valid strings into the array.
        unsafe { str::from_utf8_unchecked(self.arr.as_slice()) }
    }

}

