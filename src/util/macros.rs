/// Prints a format string to a device.
#[macro_export]
macro_rules! print {
    ($dst:expr, $fmt:expr) => ({
        use core::fmt::Write;
        let _ = write!($dst, $fmt);
    });
    ($dst:expr, $fmt:expr, $($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($dst, $fmt, $($arg)*);
    });
}

/// Prints a format string line to a device.
#[macro_export]
macro_rules! println {
    ($dst:expr) => ({
        use core::fmt::Write;
        let _ = write!($dst, "\n");
    });
    ($dst:expr, $fmt:expr) => ({
        use core::fmt::Write;
        let _ = write!($dst, concat!($fmt, "\n"));
    });
    ($dst:expr, $fmt:expr, $($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($dst, concat!($fmt, "\n"), $($arg)*);
    });
}

/// Returns the value of a linker symbol. We use a macro to ensure that we always use the ADDRESS
/// of the symbol and not the VALUE of the symbol because that is how linker symbols work!
#[macro_export]
macro_rules! linker_sym {
    ($sym:ident) => ({ 
        extern { static $sym: (); }; 
        &$sym as *const () as usize 
    });
}

