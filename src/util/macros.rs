#[macro_export]
macro_rules! print {
    (($arg:tt)*) => ({
        use core::fmt::Write;
        use io;
        let _ = write!(io::COM1, $($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    () => ({
        use core::fmt::Write;
        use io;
        let _ = write!(io::COM1, "\n");
    });
    ($fmt:expr) => ({
        use core::fmt::Write;
        use io;
        let _ = write!(io::COM1, concat!($fmt, "\n"));
    });
    ($fmt:expr, $($arg:tt)*) => ({
        use core::fmt::Write;
        use io;
        let _ = write!(io::COM1, concat!($fmt, "\n"), $($arg)*);
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
