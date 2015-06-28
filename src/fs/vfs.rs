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
use util::KernResult;
use util::KernError::*;
logger_init!(Trace);

/// A virtual file system.
pub struct VFS {
    root: Rc<VFSNode>
}

impl VFS {
    pub fn new() -> KernResult<VFS> {
        let node = try!(VFSNode::new().and_then(Box::new));
        let root = Rc::new(node);
        Ok(VFS { root: root })
    }
}

impl FileSystem for VFS {
    fn root_node(&self) -> KernResult<Rc<Node>> {
        Ok(self.root.clone())
    }
}

struct VFSFileIter<'a> {
    lock: ReaderGuardMap<'a, HashMap<str, VFSEntry>, KeyIter<'a, str, VFSEntry>>,
}

impl<'a> Iterator for VFSFileIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.lock.next()
    }
}

struct VFSNode {
    rc: AtomicUsize,
    alive: bool,
    entries: RWLock<HashMap<str, VFSEntry>>
}

impl VFSNode {
    pub fn new() -> KernResult<VFSNode> {
        let map = try!(HashMap::new());
        Ok(VFSNode {
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
    
    fn list<'a>(&'a self) -> KernResult<Box<Iterator<Item=&'a str> + 'a>> {
        let lock = self.entries.lock_reader();
        let boxed = try!(Box::new(VFSFileIter {
            lock: lock.map(|map| map.iter_keys())
        }));
        Ok(boxed)
    }

    fn count(&self) -> usize {
        assert!(self.alive);
        self.entries.lock_reader().count()
    }

    fn open_file(&self, file: &str) -> KernResult<Box<File>> {
        trace!("opening file {}", file);
        assert!(self.alive);
        let entries = self.entries.lock_reader();
        match entries.lookup(file) {
            Some(&VFSEntry::File{ ref file, .. }) => {
                let clone = try!(file.clone());  
                let boxed = try!(Box::new(clone));
                Ok(boxed)
            }
            _ => Err(NoSuchFile)
        }
        
    }

    fn open_node(&self, node: &str) -> KernResult<Rc<Node>> {
        trace!("opening node {}", node);
        assert!(self.alive);
        match self.entries.lock_reader().lookup(node) {
            Some(&VFSEntry::Node { ref node, .. }) => {
                Ok(node.clone())
            }
            Some(&VFSEntry::Mount { ref fs, .. }) => {
                fs.root_node()
            }
            _ => Err(NoSuchDirectory)
        }
    }

    fn make_file(&self, name: String) -> KernResult<()> {
        trace!("making file {}", name);
        assert!(self.alive);
        let mut lock = self.entries.lock_writer();
        if lock.contains(name.as_str()) {
            Err(FileExists)
        } else {
            let file = try!(VFSFile::new());
            let entry = VFSEntry::File { name: name, file: file, link: DoubleLink::new() };
            let entry = try!(Box::new(entry));
            assert!(lock.insert(entry).is_none());
            Ok(())
        }
    }

    fn make_node(&self, name: String) -> KernResult<()> {
        trace!("making node {}", name);
        assert!(self.alive);
        let mut lock = self.entries.lock_writer();
        if lock.contains(name.as_str()) {
            trace!("failed, {} already exists", name);
            Err(DirectoryExists)
        } else {
            let node = try!(VFSNode::new().and_then(Box::new).map(Rc::new));
            let entry = VFSEntry::Node { name: name, node: node, link: DoubleLink::new() };
            let entry = try!(Box::new(entry));
            assert!(lock.insert(entry).is_none());
            Ok(())
        }
    }

    fn mount(&self, name: String, fs: Box<FileSystem>) -> KernResult<()> {
        let mut lock = self.entries.lock_writer();
        if lock.contains(name.as_str()) {
            Err(DirectoryExists)
        } else {
            let entry = VFSEntry::Mount { name: name, fs: fs, link: DoubleLink::new() };
            let entry = try!(Box::new(entry));
            assert!(lock.insert(entry).is_none());
            Ok(())
        }
    }

}


enum VFSEntry {
    Node { name: String, node: Rc<VFSNode>, link: DoubleLink<VFSEntry> },
    File { name: String, file: VFSFile, link: DoubleLink<VFSEntry> },
    Mount { name: String, fs: Box<FileSystem>, link: DoubleLink<VFSEntry> },
}

impl HasKey<str> for VFSEntry {
    fn get_key(&self) -> &str {
        match self {
            &VFSEntry::Node { ref name, .. } => name.as_str(),
            &VFSEntry::File { ref name, .. } => name.as_str(),
            &VFSEntry::Mount { ref name, .. } => name.as_str(),
        }
    }
}

impl HasDoubleLink<VFSEntry> for VFSEntry {
    fn dlink(&self) -> &DoubleLink<VFSEntry> {
        match self {
            &VFSEntry::Node { ref link, .. } => link,
            &VFSEntry::File { ref link, .. } => link,
            &VFSEntry::Mount { ref link, .. } => link,
        }
    }

    fn dlink_mut (&mut self) -> &mut DoubleLink<VFSEntry> {
        match self {
            &mut VFSEntry::Node { ref mut link, .. } => link,
            &mut VFSEntry::File { ref mut link, .. } => link,
            &mut VFSEntry::Mount { ref mut link, .. } => link,
        }
    }
}

struct VFSFile {
    data: DynArray<u8>
}

impl VFSFile {

    pub fn new() -> KernResult<VFSFile> {
        let dyn = try!(DynArray::new(32));
        Ok(VFSFile {
            data: dyn
        })
    }

    pub fn clone(&self) -> KernResult<VFSFile> {
        let dynclone = try!(self.data.clone());
        Ok(VFSFile {
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
