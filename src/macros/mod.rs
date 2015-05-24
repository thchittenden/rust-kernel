#![crate_name="macros"]
#![crate_type="rlib"]
#![feature(no_std)]
#![no_std]

#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => ( loop { } );
}

#[macro_export]
macro_rules! print {
    ($dst:expr, $($arg:tt)*) => ({
        use core::Result::result::{Ok, Err};
        use core::fmt::Write;
        match format_args!(|args| $dst.write_fmt(args), $($arg)*) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from Write"),
        }
    });
}

#[macro_export]
macro_rules! println {
    ($dst:expr) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        match $dst.write_fmt(format_args!("\n")) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from Write"),
        }
    });
    ($dst:expr, $fmt:expr) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        match $dst.write_fmt(format_args!(concat!($fmt, "\n"))) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from Write"),
        }
    });
    ($dst:expr, $fmt:expr, $($arg:tt)*) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        match $dst.write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*)) {
            Ok (_) => { },
            Err(_) => panic!("PANIC: got error from Write"),
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
