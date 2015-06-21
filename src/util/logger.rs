use core::marker::Sync;
use core::fmt;

/// Log levels available.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Logs everything. Trace is used for logging fine grained debugging events.
    Trace = 1,
    /// Logs debugging events at a higher level than trace.
    Debug = 2,
    /// Logs events that the user may be curious about.
    Info = 3,
    /// Logs events that may prevent the system from functioning optimally but that do not prevent
    /// it from running.
    Warn = 4,
    /// Logs events that may prevent the system from continuing to function.
    Error = 5,
    /// Logs nothing. 
    Quiet = 6,
}

// We promise someone will implement this hook in Rust.
#[allow(improper_ctypes)]
extern {
    fn logger_hook(s: &str) -> fmt::Result;
}

/// The log writer struct that all log messages are funneled through. This hooks into a library at
/// a higher level in order to perform the IO.
pub struct LogWriter;

impl fmt::Write for LogWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe { logger_hook(s) }
    }
}

// We promise logger_hook is safe, right?
unsafe impl Sync for LogWriter { }
pub static mut LOG: LogWriter = LogWriter;

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

#[macro_export]
macro_rules! logger_log {
    ($lvl:ident, $fmt:expr) => ({
        if logger_level_enabled!($crate::logger::LogLevel::$lvl) {
            unsafe { println!($crate::logger::LOG, $fmt) };
        }
    });
    ($lvl:ident, $fmt:expr, $($arg:tt)*) => ({
        if logger_level_enabled!($crate::logger::LogLevel::$lvl) {
            unsafe { println!($crate::logger::LOG, $fmt, $($arg)*) };
        }
    })
}

#[macro_export]
macro_rules! trace {
    ($fmt:expr) => ( logger_log!(Trace, concat!("TRACE: ", $fmt)) );
    ($fmt:expr, $($arg:tt)*) => ( logger_log!(Trace, concat!("TRACE: ", $fmt), $($arg)*) );
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr) => ( logger_log!(Debug, concat!("DEBUG: ", $fmt)) );
    ($fmt:expr, $($arg:tt)*) => ( logger_log!(Debug, concat!("DEBUG: ", $fmt), $($arg)*) );
}

#[macro_export]
macro_rules! info {
    ($fmt:expr) => ( logger_log!(Info, concat!("INFO: ", $fmt)) );
    ($fmt:expr, $($arg:tt)*) => ( logger_log!(Info, concat!("INFO: ", $fmt), $($arg)*) );
}

