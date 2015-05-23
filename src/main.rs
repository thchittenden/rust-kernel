#![feature(lang_items, no_std, start)]
#![no_std]

#[lang="sized"]
trait Sized { }

#[lang="copy"]
trait Copy { }

#[start]
fn main(argc: isize, argv: *const *const u8) -> isize {
    42
}


//#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
//#[lang = "eh_personality"] extern fn eh_personality() {}
//#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
