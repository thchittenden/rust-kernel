use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::atomic::AtomicUsize;
use collections::hashmap::{HashMap, HasKey};
use collections::dynarray::DynArray;
use collections::link::{HasDoubleLink, DoubleLink};
use collections::string::String;
use collections::vec::Vec;
use sync::rwlock::RWLock;
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

struct VFSCursor {
    cur_dir: Rc<VFSNode>
}

impl FileCursor for VFSCursor {
    fn open(&self, path: Path) -> Option<Box<File>> {
        unimplemented!()
    }
    fn list(&self) -> Option<Vec<&String>> {
        unimplemented!()
    }
    fn mkdir(&self, name: &str) -> Option<Box<FileCursor>> {
        unimplemented!()
    }
    fn cd(&mut self, path: Path) -> bool {
        unimplemented!()
    }
}

struct VFSNode {
    rc: AtomicUsize,
    alive: bool,
    entries: RWLock<HashMap<String, VFSEntry>>
}

impl VFSNode {
    fn new() -> Option<VFSNode> {
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

