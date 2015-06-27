/// Returns the value of a linker symbol. We use a macro to ensure that we always use the ADDRESS
/// of the symbol and not the VALUE of the symbol because that is how linker symbols work!
#[macro_export]
macro_rules! linker_sym {
    ($sym:ident) => ({ 
        extern { static $sym: (); }; 
        &$sym as *const () as usize 
    });
}

/// The libstd try! macro instead of the gimped libcore version.
#[macro_export]
macro_rules! try {
    ($exp:expr) => ({
        use core::result::Result::{Ok, Err};
        use core::convert::From;
        match $exp {
            Ok(v) => v,
            Err(e) => return Err(From::from(e))
        }
    }); 
    ($exp:expr, $ex:expr) => ({
        use core::result::Result::{Ok, Err};
        use core::convert::From;
        use $crate::KernErrorEx;
        match $exp {
            Ok(v) => v,
            Err(e) => return Err(KernErrorEx { err: From::from(e), ex: $ex })
        }
    });
}

#[macro_export]
macro_rules! try_op {
    ($exp:expr) => ({
        use core::option::Option::{Some, None};
        match $exp {
            None => return None,
            Some(val) => val
        }
    })
}

#[macro_export]
macro_rules! align {
    ($val:expr, $align:expr) => ({
        assert!($crate::is_pow2($align));
        $val & !($align - 1)
    });
}

#[macro_export]
macro_rules! align_up {
    ($val:expr, $align:expr) => ({
        assert!($crate::is_pow2($align));
        ($val + $align - 1) & !($align - 1)
    });
}

#[macro_export]
macro_rules! is_aligned {
    ($val:expr, $align:expr) => ({
        align!($val, $align) == $val
    });
}

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

