#![crate_name="rt"]
#![crate_type="rlib"]
#![feature(no_std,core,core_prelude,lang_items)]
#![no_std]
#![no_builtins]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate io;

use core::prelude::*;
use core::fmt::{Arguments, Write};
use core::num::Wrapping;
use io::console::{Console, Color};

// This is our panic function. It must be declared "extern" or arguments
// will be mangled on the stack.
#[allow(unused_must_use)]
#[lang = "panic_fmt"] 
#[no_mangle]
pub extern fn rust_begin_unwind(args: Arguments, file: &str, line: usize) -> ! { 
    
    // Construct a new console and clear it. This is important in case our 
    // console was corrupted.
    let mut con = Console::new();
    con.clear();
    con.set_color(Color::LightRed, Color::Black);

    // Print the panic messages.
    con.write_fmt(format_args!("PANIC ({}, {}):\n\t", file, line));
    con.write_fmt(args);

    // Don't return.
    loop {} 
}

// Converts a C style string (*const u8) to a rust &str
macro_rules! cstr {
    ($s:expr) => ({
        use core::slice;
        use core::str;
        str::from_utf8_unchecked(slice::from_raw_parts($s, strlen($s) as usize))
    })
}

// This is used by LMM.
#[no_mangle]
pub unsafe extern fn __assert_fail(msg: *const u8, file: *const u8, line: usize, func: *const u8) -> ! {
    rust_begin_unwind(format_args!("{}: {}", cstr!(func), cstr!(msg)), cstr!(file), line);
}

#[no_mangle]
pub unsafe fn strlen(s: *const u8) -> isize {
    let mut len = 0;
    while *s.offset(len) != 0 {
        len += 1;
    }
    len
}

#[lang = "stack_exhausted"]
extern fn stack_exhausted() {}

#[lang = "eh_personality"] 
extern fn eh_personality() {}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn _Unwind_Resume(_: *mut ()) { }
