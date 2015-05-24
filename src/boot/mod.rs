#![crate_name="boot"]
#![crate_type="rlib"]
#![feature(no_std,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate macros;
extern crate console;
extern crate mem;

mod multiboot;

use multiboot::MultibootHeader;

#[no_mangle]
pub extern "C" fn kernel_main (hdr: &MultibootHeader) -> ! {
    println!("hello from a brand new kernel");
    println!("testing, {}, {}, {}, {}...", 1, 2, 3, 4);
    println!("{:?}", hdr);

    // TODO remove kernel!
    hdr.walk_mmap(|base, len| println!("({}:{})", base, len));
    //hdr.walk_mmap(mem::phys::add_range);

    // Don't return.
    loop { }
}
