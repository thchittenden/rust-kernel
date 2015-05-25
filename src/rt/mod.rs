#![crate_name="rt"]
#![crate_type="rlib"]
#![feature(no_std,core,lang_items)]
#![no_std]
#![no_builtins]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
extern crate console;

use core::fmt::{Arguments, Write};
use console::{Console, Color};

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

#[no_mangle]
pub unsafe fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0 .. len as isize {
        *dst.offset(i) = *src.offset(i);
    }
}

#[no_mangle]
pub unsafe fn memset(dst: *mut u8, val: isize, len: usize) {
    for i in 0 .. len as isize {
        *dst.offset(i) = val as u8;
    }
}

#[no_mangle]
pub unsafe fn memcmp(p1: *const u8, p2: *const u8, len: usize) -> isize {
    for i in 0 .. len as isize {
        let diff = (*p1.offset(i) - *p2.offset(i)) as i8;
        if diff != 0 {
            return diff as isize;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe fn __udivdi3() { }

#[no_mangle]
pub unsafe fn __umoddi3() { }

#[lang = "stack_exhausted"]
extern fn stack_exhausted() {}

#[lang = "eh_personality"] 
extern fn eh_personality() {}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn _Unwind_Resume(_: *mut ()) { }
