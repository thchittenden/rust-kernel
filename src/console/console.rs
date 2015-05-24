use core::fmt::{Write, Error};
use core::result::Result;
use core::result::Result::Ok;
use core::str::StrExt;

use color::Color;

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

impl Console {

    pub fn new() -> Console {
        Console {
            rows: 25,
            cols: 80,
            row: 0,
            col: 0,
            color_fg: Color::White,
            color_bg: Color::Black,
            base: 0xB8000 as *mut u16,
        }
    }

    pub fn clear(&mut self) {
        self.col = 0;
        self.row = 0;
        for i in 0 .. self.cols * self.rows {
            unsafe { *self.base.offset(i) = 0x0 }
        }
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_fg = fg;
        self.color_bg = bg;
    }
    
    pub fn putc(&mut self, c: char) {
        //assert!(self.row < self.rows); 
        //assert!(self.col < self.cols);

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
        }
    }
}


impl Write for Console {
  
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        // TODO, need a mutex around this.
        for c in s.chars() {
            self.putc(c)
        }
        Ok(())
    }

}

