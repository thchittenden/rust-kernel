use alloc;

mod boxes;
mod rc;
mod vec;
mod map;
mod string;
mod vfs;
mod hashmap;
mod slist;
mod mem;

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
    vfs::test();
    slist::test();
    mem::test();
    let free_end = alloc::get_free_space();

    // VFS may "leak" bytes so perform it after we check for leaks.

    trace!("\n==== ENDING TESTS ====");
    trace!("leaked {} bytes\n", free_start - free_end);
}

pub fn vfs_shell() -> ! {
    vfs::vfs_shell()
}
