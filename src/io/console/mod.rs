//!
//! This module contains an interface for interacting with the VGA console. All operations are
//! performed through a `SafeConsole` which uses a mutex to protect a regular `Console`.
//!

use self::console::SAFE_CONSOLE_INIT;

pub use self::console::SafeConsole;
pub use self::console::Console;
pub use self::color::*; 

/// The interface to the VGA console.
pub mod console;

/// Definitions of the colors used in the console.
pub mod color;

/// The system-wide console.
pub static CON: SafeConsole = SAFE_CONSOLE_INIT;

