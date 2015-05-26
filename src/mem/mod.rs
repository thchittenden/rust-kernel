#![crate_name="mem"]
#![crate_type="rlib"]
#![feature(no_std,core,step_by,unique)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate sync;
extern crate alloc;
extern crate console;

pub mod phys;
pub mod virt;

pub fn init() {
    phys::init();
    virt::init();
    direct_map_kernel();
    //enable_paging();
}

// This function sets up a page directory and the necessary page tables in 
// order to direct map the kernel.
fn direct_map_kernel() {
    
}


