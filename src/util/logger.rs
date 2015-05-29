#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
    Quiet = 6,
}

// If there's a better way to do this... I'd like to know.
#[cfg(LOG_LEVEL="trace")]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Trace;
#[cfg(LOG_LEVEL="debug")]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Debug;
#[cfg(LOG_LEVEL="info")]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Info;
#[cfg(LOG_LEVEL="warn")]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Warn;
#[cfg(LOG_LEVEL="error")]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Error;
#[cfg(LOG_LEVEL="quiet")]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Quiet;
#[cfg(not(any(LOG_LEVEL="trace",LOG_LEVEL="debug",LOG_LEVEL="info",LOG_LEVEL="warn",LOG_LEVEL="error",LOG_LEVEL="quiet")))]
pub const GLOBAL_LOG_LEVEL: LogLevel = LogLevel::Trace;

/// Initializes the module-level logger at the given log level.
#[macro_export]
macro_rules! logger_init {
    () => { const LOG_LEVEL: $crate::logger::LogLevel = $crate::logger::GLOBAL_LOG_LEVEL; };
    ($lvl:ident) => { const LOG_LEVEL: $crate::logger::LogLevel = $crate::logger::LogLevel::$lvl; };
}

/// Returns whether a logger level is enabled or not.
#[macro_export]
macro_rules! logger_level_enabled {
    ($lvl:path) => { $crate::logger::GLOBAL_LOG_LEVEL <= $lvl && LOG_LEVEL <= $lvl };
}

#[cfg(LOG_DEVICE="console")]
#[macro_export]
macro_rules! logger_get_device {
    () => ({ 
        use console::CON;
        &CON 
    });
}

#[cfg(not(LOG_DEVICE="console"))]
#[macro_export]
macro_rules! logger_get_device {
    () => ({ 
        use io::COM1;
        &COM1 
    });
}

#[macro_export]
macro_rules! trace {
    ($fmt:expr) => ({
        if logger_level_enabled!($crate::logger::LogLevel::Trace) {
            println!(logger_get_device!(), concat!("TRACE: ", $fmt));
        }
    });
    ($fmt:expr, $($arg:tt)*) => ({
        if logger_level_enabled!($crate::logger::LogLevel::Trace) {
            println!(logger_get_device!(), concat!("TRACE: ", $fmt), $($arg)*);
        }
    });
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr) => ({
        if logger_level_enabled!($crate::logger::LogLevel::Debug) {
            println!(logger_get_device!(), concat!("DEBUG: ", $fmt));
        }
    });
    ($fmt:expr, $($arg:tt)*) => ({
        if logger_level_enabled!($crate::logger::LogLevel::Debug) {
            println!(logger_get_device!(), concat!("DEBUG: ", $fmt), $($arg)*);
        }
    });
}
