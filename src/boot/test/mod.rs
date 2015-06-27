use alloc;

mod boxes;
mod rc;
mod vec;
mod map;
mod string;
mod vfs;
logger_init!(Trace);

pub fn test_all() {
    trace!("\n\n ==== BEGINNING TESTS ====\n");
    let free_start = alloc::get_free_space();

    boxes::test();
    rc::test();
    vec::test();
    map::test();
    string::test();
    vfs::test();

    trace!("\n==== ENDING TESTS ====");
    let free_end = alloc::get_free_space();
    trace!("leaked {} bytes\n", free_start - free_end);
}
