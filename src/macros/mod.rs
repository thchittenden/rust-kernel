#![feature(no_std)]
#![no_std]

#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => ( fail!($($arg)*) );
}

#[macro_export]
macro_rules! print {
    ($dst:expr, $($arg:tt)*) => ({
        use core::result::{Ok, Err};
        use core::fmt::Write;
        match format_args!(|args| $dst.write_fmt(args), $($arg)*) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from FormatWriter"),
        }
    });
}

#[macro_export]
macro_rules! println {
    ($dst:expr) => ({
        use core::result::{Ok, Err};
        use core::fmt::FormatWriter;
        match format_args!(|args| $dst.write_fmt(args), "\n") {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from FormatWriter"),
        }
    });
    ($dst:expr, $fmt:expr) => ({
        use core::result::{Ok, Err};
        use core::fmt::FormatWriter;
        match format_args!(|args| $dst.write_fmt(args), concat!($fmt, "\n")) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from FormatWriter"),
        }
    });
    ($dst:expr, $fmt:expr, $($arg:tt)*) => ({
        use core::result::{Ok, Err};
        use core::fmt::FormatWriter;
        match format_args!(|args| $dst.write_fmt(args), concat!($fmt, "\n"), $($arg)*) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from FormatWriter"),
        }
    });
}

#[macro_export]
macro_rules! linker_sym {
    ($sym:ident) => ({ 
        use core::mem::transmute;
        extern { static $sym: (); }; 
        unsafe { transmute(&$sym) } // have to take address here because linker symbol!
    });
}

#[macro_export]
macro_rules! unimplemented {
    () => ( panic!("PANIC: unimplemented!") );
}
