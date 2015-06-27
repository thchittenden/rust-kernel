use alloc;

mod boxes;
mod rc;
mod vec;
mod map;
mod string;
mod vfs;
mod hashmap;
logger_init!(Trace);

pub fn test_all() {
    trace!("\n\n ==== BEGINNING TESTS ====\n");

    let free_start = alloc::get_free_space();
    boxes::test();
    rc::test();
    vec::test();
    map::test();
    string::test();
    hashmap::test();
    let free_end = alloc::get_free_space();

    // VFS may "leak" bytes so perform it after we check for leaks.
    vfs::test();

    trace!("\n==== ENDING TESTS ====");
    trace!("leaked {} bytes\n", free_start - free_end);
}
