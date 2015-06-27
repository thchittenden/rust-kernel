use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::atomic::AtomicUsize;
use collections::hashmap::{HashMap, HasKey, KeyIter};
use collections::dynarray::DynArray;
use collections::link::{HasDoubleLink, DoubleLink};
use collections::string::String;
use sync::rwlock::{ReaderGuardMap, RWLock};
use super::{Path, File, FileCursor, FileSystem};

/// A virtual file system.
pub struct VFS {
    root: Rc<VFSNode>
}

impl VFS {
    pub fn new() -> Option<VFS> {
        let root = Rc::new(try_op!(Box::new(try_op!(VFSNode::new()))));
        Some(VFS { root: root })
    }
}

impl FileSystem for VFS {
    fn create_cursor(&self) -> Option<Box<FileCursor>> {
        // We must "try" to unwrap this and rewrap it in order to coerce the upcast from VFSCursor
        // to FileCursor.
        Some(try_op!(Box::new(VFSCursor {
            cur_dir: self.root.clone() 
        })))

    }
}

struct VFSFileIter<'a> {
    lock: ReaderGuardMap<'a, HashMap<String, VFSEntry>, KeyIter<'a, String, VFSEntry>>,
}

impl<'a> Iterator for VFSFileIter<'a> {
    type Item = &'a String;
    fn next(&mut self) -> Option<&'a String> {
        self.lock.next()
    }
}

struct VFSCursor {
    cur_dir: Rc<VFSNode>
}

impl FileCursor for VFSCursor {
    fn open(&self, file: String) -> Option<Box<File>> {
        assert!(self.cur_dir.alive);
        let entries = self.cur_dir.entries.lock_reader();
        match entries.lookup(&file) {
            None => None,
            Some(&VFSEntry::Node{ .. }) => None,
            Some(&VFSEntry::File{ ref file, .. }) => {
                let clone = try_op!(file.clone());  
                let boxed = try_op!(Box::new(clone));
                Some(boxed)
            }
        }
        
    }

    fn list<'a>(&'a self) -> Option<Box<Iterator<Item=&'a String> + 'a>> {
        let lock = self.cur_dir.entries.lock_reader();
        let boxed = try_op!(Box::new(VFSFileIter {
            lock: lock.map(|map| map.iter_keys())
        }));
        Some(boxed)
    }

    fn mkdir(&self, _: &str) -> Option<Box<FileCursor>> {
        unimplemented!()
    }

    fn cd(&mut self, _: Path) -> bool {
        unimplemented!()
    }
}

struct VFSNode {
    rc: AtomicUsize,
    alive: bool,
    entries: RWLock<HashMap<String, VFSEntry>>
}

impl VFSNode {
    pub fn new() -> Option<VFSNode> {
        let map = try_op!(HashMap::new());
        Some(VFSNode {
            rc: AtomicUsize::new(0),
            alive: true,
            entries: RWLock::new(map)
        })
    }
}

impl HasRc for VFSNode {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}


enum VFSEntry {
    Node { name: String, node: Rc<VFSNode>, link: DoubleLink<VFSEntry> },
    File { name: String, file: VFSFile, link: DoubleLink<VFSEntry> },
}

impl HasKey<String> for VFSEntry {
    fn get_key(&self) -> &String {
        match self {
            &VFSEntry::Node { ref name, .. } => name,
            &VFSEntry::File { ref name, .. } => name,
        }
    }
}

impl HasDoubleLink<VFSEntry> for VFSEntry {
    fn dlink(&self) -> &DoubleLink<VFSEntry> {
        match self {
            &VFSEntry::Node { ref link, .. } => link,
            &VFSEntry::File { ref link, .. } => link,
        }
    }

    fn dlink_mut (&mut self) -> &mut DoubleLink<VFSEntry> {
        match self {
            &mut VFSEntry::Node { ref mut link, .. } => link,
            &mut VFSEntry::File { ref mut link, .. } => link,
        }
    }
}

struct VFSFile {
    data: DynArray<u8>
}

impl VFSFile {

    pub fn new() -> Option<VFSFile> {
        let dyn = try_op!(DynArray::new(32));
        Some(VFSFile {
            data: dyn
        })
    }

    pub fn clone(&self) -> Option<VFSFile> {
        let dynclone = try_op!(self.data.clone());
        Some(VFSFile {
            data: dynclone
        })
    }

}

impl File for VFSFile {
    
    unsafe fn read(&self, into: usize, offset: usize, count: usize) -> usize {
        let into_ptr = into as *mut u8;
        for i in 0..count {
            if offset + i >= self.data.len() {
                return i;
            } else {
                *into_ptr.offset(i as isize) = self.data[offset + i];
            }
        }
        count
    }

    unsafe fn write(&mut self, from: usize, offset: usize, count: usize) -> usize {
        // Try to expand the file if needed.
        if offset + count > self.data.len() {
            // We can ignore whether this succeeds or not because we will respect data.len()
            // regardless.
            let _ = self.data.resize(offset + count);
        }
        
        // Write the data.
        let from_ptr = from as *const u8;
        for i in 0..count {
            if offset + i >= self.data.len() {
                return i;
            } else {
                self.data[offset + i] = *from_ptr.offset(i as isize);
            }
        }
        count
    }

}
