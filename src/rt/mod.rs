#![crate_name="rt"]
#![crate_type="rlib"]
#![feature(no_std,core,lang_items)]
#![no_std]
#![no_builtins]
extern crate core;

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }

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
#[allow(non_snake_case)]
pub unsafe fn _Unwind_Resume(_: *mut ()) { }
