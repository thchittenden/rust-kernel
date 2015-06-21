use core::prelude::*;
use core::{fmt, str};
use core::hash::{Hash, Hasher};
use super::vec::Vec;

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

    fn make_dynamic(&mut self) -> bool {
        *self = match self {
            &mut DynamicString(_) => return true,
            &mut StaticString(ref s) => { 
                if let Some(ds) = DynString::from_str(s) {
                    DynamicString(ds)
                } else {
                    return false
                }
            }
        };
        true
    }

    pub fn append(&mut self, s: &str) -> bool {
        if self.is_static() {
            self.make_dynamic() && self.append(s)
        } else {
            match self {
                &mut StaticString(_) => unreachable!(),
                &mut DynamicString(ref mut ds) => ds.append(s)
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            &StaticString(ref s) => s,
            &DynamicString(ref ds) => ds.as_ref()
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
        if self.append(s) {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

/// A dynamically growable string.
pub struct DynString {
    arr: Vec<u8>,
}

impl DynString {
    
    pub fn new() -> Option<DynString> {
        let vec = try_op!(Vec::new(16));
        Some(DynString { arr: vec })
    }

    pub fn from_str(s: &str) -> Option<DynString> {
        let mut vec = try_op!(Vec::new(s.len()));
        for byte in s.as_bytes() {
            assert!(vec.push(*byte).is_ok())
        }
        Some(DynString { arr: vec })
    }

    pub fn append(&mut self, s: &str) -> bool {
        for byte in s.as_bytes() {
            if self.arr.push(*byte).is_err() {
                return false;
            }
        }
        true
    }

    pub fn as_ref(&self) -> &str {
        // We don't really know this is safe... TODO 
        unsafe { str::from_utf8_unchecked(self.arr.as_slice()) }
    }

}

