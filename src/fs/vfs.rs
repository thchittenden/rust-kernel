use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::atomic::AtomicUsize;
use collections::hashmap::{HashMap, HasKey, KeyIter};
use collections::dynarray::DynArray;
use collections::link::{HasDoubleLink, DoubleLink};
use collections::string::String;
use sync::rwlock::{ReaderGuardMap, RWLock};
use super::{Node, File, FileSystem};
use path::Path;

/// A virtual file system.
pub struct VFS {
    root: Rc<VFSNode>
}

impl VFS {
    pub fn new() -> Option<VFS> {
        let node = try_op!(VFSNode::new().and_then(Box::new));
        let root = Rc::new(node);
        Some(VFS { root: root })
    }
}

impl FileSystem for VFS {
    fn root_node(&self) -> Option<Rc<Node>> {
        // We must "try" to unwrap this and rewrap it in order to coerce the upcast from VFSCursor
        // to FileCursor.
        Some(self.root.clone())
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

impl Node for VFSNode {
    
    fn list<'a>(&'a self) -> Option<Box<Iterator<Item=&'a String> + 'a>> {
        let lock = self.entries.lock_reader();
        let boxed = try_op!(Box::new(VFSFileIter {
            lock: lock.map(|map| map.iter_keys())
        }));
        Some(boxed)
    }

    fn open_file(&self, file: String) -> Option<Box<File>> {
        assert!(self.alive);
        let entries = self.entries.lock_reader();
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

    fn open_node(&self, node: String) -> Option<Rc<Node>> {
        assert!(self.alive);
        match self.entries.lock_reader().lookup(&node) {
            None => None,
            Some(&VFSEntry::File { .. }) => None,
            Some(&VFSEntry::Node { ref node, .. }) => {
                Some(node.clone())
            }
        }
    }

    fn make_file(&self, name: String) -> bool {
        assert!(self.alive);
        let mut lock = self.entries.lock_writer();
        if lock.contains(&name) {
            return false;
        } else {
            let file = try_or!(VFSFile::new(), return false);
            let entry = VFSEntry::File { name: name, file: file, link: DoubleLink::new() };
            let entry = try_or!(Box::new(entry), return false);
            lock.insert(entry);
            return true;
        }
    }

    fn make_node(&self, name: String) -> bool {
        assert!(self.alive);
        let mut lock = self.entries.lock_writer();
        if lock.contains(&name) {
            return false;
        } else {
            let node = try_or!(VFSNode::new().and_then(Box::new).map(Rc::new), return false);
            let entry = VFSEntry::Node { name: name, node: node, link: DoubleLink::new() };
            let entry = try_or!(Box::new(entry), return false);
            lock.insert(entry);
            return true;
        }
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
