#![crate_name="alloc"]
#![crate_type="rlib"]
#![feature(no_std,lang_items,unique,core)]
#![no_std]

#[macro_use] extern crate core;
#[macro_use] extern crate util;
#[macro_use] extern crate sync;
extern crate console;
logger_init!(Trace);

pub mod boxed;

mod naive;

use core::prelude::*;
use core::ptr::Unique;
use sync::mutex::Mutex;
use naive::{NaiveAllocator, NAIVE_ALLOCATOR_INIT};

trait Allocator {

    fn allocate<T>(&mut self, elem: T) -> Option<Unique<T>>;

    fn deallocate<T>(&mut self, elem: Unique<T>);

}

static ALLOCATOR: Mutex<NaiveAllocator> = static_mutex!(NAIVE_ALLOCATOR_INIT);

pub fn init() {
    let heap_start = linker_sym!(__heap_start);
    let heap_end = linker_sym!(__heap_end);
    trace!("initializing allocator ({:x}-{:x})", heap_start, heap_end); 
    ALLOCATOR.lock().unwrap().init(heap_start, heap_end);
}

pub fn allocate<T>(elem: T) -> Option<Unique<T>> {
    ALLOCATOR.lock().unwrap().allocate(elem)
}

pub fn deallocate<T>(elem: Unique<T>) {
    ALLOCATOR.lock().unwrap().deallocate(elem)
}
