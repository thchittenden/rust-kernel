#![crate_name="macros"]
#![crate_type="rlib"]
#![feature(no_std)]
#![no_std]

#[macro_export]
macro_rules! print {
    ($dst:expr, $($arg:tt)*) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        write!($dst, $($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    ($dst:expr) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        write!($dst, "\n");
    });
    ($dst:expr, $fmt:expr) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        write!($dst, concat!($fmt, "\n"));
    });
    ($dst:expr, $fmt:expr, $($arg:tt)*) => ({
        use core::result::Result::{Ok, Err};
        use core::fmt::Write;
        write!($dst, concat!($fmt, "\n"), $($arg)*);
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
