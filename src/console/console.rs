//!
//! This module contains the definition of the `Console` and `SafeConsole` objects which permit
//! interaction with the VGA memory area located at `0xB8000`. 
//!
//! This console supports a 80x25 screen area and 16 different colors for both the background and
//! foreground.
//!
use core::fmt::{Write, Error, Arguments};
use core::result::Result;
use core::result::Result::Ok;
use core::str::StrExt;

use color::Color;

use mutex::Mutex;

/// A VGA console.
pub struct Console {
    // Console dimensions.
    rows: isize,
    cols: isize,

    // Current cursor position.
    row: isize,
    col: isize,
    
    // Current color.
    color_fg: Color,
    color_bg: Color,

    // Pointer into VGA memory.
    base: *mut u16,
}

pub const CONSOLE_INIT: Console = Console {
    rows: 25,
    cols: 80,
    row: 0,
    col: 0,
    color_fg: Color::White,
    color_bg: Color::Black,
    base: 0xB8000 as *mut u16 ,
};

/// A thread-safe VGA console.
pub struct SafeConsole {
    con: Mutex<Console>
}

pub const SAFE_CONSOLE_INIT: SafeConsole = SafeConsole {
    con: static_mutex!(CONSOLE_INIT)
};

impl Console {

    /// Creates a new VGA console.
    pub fn new() -> Console {
        CONSOLE_INIT
    }

    /// Clears the console to black and resets the cursor position to (0,0).
    pub fn clear(&mut self) {
        self.col = 0;
        self.row = 0;
        for i in 0 .. self.cols * self.rows {
            unsafe { *self.base.offset(i) = 0x0 }
        }
    }

    /// Sets the foreground and background colors of the console.
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_fg = fg;
        self.color_bg = bg;
    }
    
    /// Puts a character at the current cursor position and moves the cursor depending on the
    /// character. New-lines, carriage-returns and tabs are all supported for moving the cursor.
    pub fn putc(&mut self, c: char) {
        assert!(self.row < self.rows); 
        assert!(self.col < self.cols);

        let color = ((self.color_bg as u16) << 12) | ((self.color_fg as u16) << 8);
        match c {
            '\n' => { self.col = 0; self.row += 1 }, // newline
            '\r' => { self.col = 0 },                // carriage return
            '\t' => { self.col += 4 },               // tabs are 4, deal with it.
            _    => unsafe { *self.base.offset(self.row * self.cols + self.col) =  color | c as u16; self.col += 1 },
        }
        
        // Check if we exceeded any bounds.
        if self.col >= self.cols {
            // Line overflow. Go to next line.
            self.row += 1;
            self.col =  0;
        }
        if self.row >= self.rows {
            // Console overflow. Scroll.
            for i in 0 .. self.cols * (self.rows - 1) {
                unsafe { *self.base.offset(i) = *self.base.offset(i + self.cols) };
            }
            for i in self.cols * (self.rows - 1) .. self.cols * self.rows {
                unsafe { *self.base.offset(i) = 0x0 };
            }
            self.row -= 1;
        }
    }
}

  
impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.chars() {
            self.putc(c)
        }
        Ok(())
    }
}

// ALMOST Write, but since SafeConsole has interior mutability the method
// should not accept a mut reference like Write expects.
impl SafeConsole {
    
    pub fn write_str(&self, s: &str) -> Result<(), Error> {
        let mut con = self.con.lock().unwrap();
        con.write_str(s)
    }

    pub fn write_fmt(&self, args: Arguments) -> Result<(), Error> {
        let mut con = self.con.lock().unwrap();
        con.write_fmt(args)
    }

}


