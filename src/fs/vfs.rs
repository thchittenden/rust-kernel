//! An in-memory virtual file system.
//!
//! This file system lives at the root of our kernel. Directories are implemented as hashmaps from
//! Strings to VFSEntries. Synchronization is performed at each directory using a reader-writer
//! lock.
//! 
//! Every directory (VFSNode) additionally contains a reference counted pointer to its parent.
//! While this introduces a cycle in reference-counted graph, this is ok because we will manually
//! tear down the graph when removing directories. It is illegal to remove a directory unless it is
//! empty and thus there will be no backreferences to it when it is removed.
//!
use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc};
use core::prelude::*;
use core::atomic::AtomicUsize;
use collections::hashmap::{HashMap, HasKey, KeyIter};
use collections::dynarray::DynArray;
use collections::link::{HasDoubleLink, DoubleLink};
use collections::string::String;
use sync::rwlock::{ReaderGuard, WriterGuard, ReaderGuardMap, RWLock};
use super::{Node, File, FileSystem};
use util::KernResult;
use util::KernError::*;
use ::PARENT_DIR;
logger_init!(Trace);

/// A virtual file system.
pub struct VFS {
    root: Rc<VFSNode>
}

impl VFS {
    pub fn new() -> KernResult<VFS> {
        let node = try!(VFSNode::new(VFSParent::Root).and_then(Box::new));
        let root = Rc::new(node);
        Ok(VFS { root: root })
    }
}

impl FileSystem for VFS {
    fn root_node(&self) -> KernResult<Rc<Node>> {
        Ok(self.root.clone())
    }
    fn set_parent(&mut self, parent: Option<Rc<Node>>) {
        let mut state = self.root.state.lock_writer();
        state.parent = match parent {
            Some(rc) => VFSParent::Linked(rc),
            None => VFSParent::Root,
        };
    }
}

struct VFSFileIter<'a> {
    lock: ReaderGuardMap<'a, VFSNodeState, KeyIter<'a, str, VFSEntry>>,
}

impl<'a> Iterator for VFSFileIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.lock.next()
    }
}

/// Possible states for a VFSNode's parent.
enum VFSParent {
    /// The parent is alive and at the contained node.
    Linked(Rc<Node>),

    /// The current node was removed from the parent.
    Unlinked,

    /// The current node has no parent because it's the root.
    Root,
}

struct VFSNodeState {
    parent: VFSParent,
    entries: HashMap<str, VFSEntry>,
}

struct VFSNode {
    rc: AtomicUsize,
    state: RWLock<VFSNodeState>,
}

impl VFSNode {
    pub fn new(parent: VFSParent) -> KernResult<VFSNode> {
        let map = try!(HashMap::new());
        Ok(VFSNode {
            rc: AtomicUsize::new(0),
            state: RWLock::new(VFSNodeState {
                parent: parent,
                entries: map,
            })
        })
    }

    /// Locks the state as a reader and checks to make sure this directory is still linked. If it
    /// is not this returns Err(DirectoryUnlinked).
    #[inline]
    fn checked_lock_reader(&self) -> KernResult<ReaderGuard<VFSNodeState>> {
        let state = self.state.lock_reader();
        match state.parent {
            VFSParent::Unlinked => Err(DirectoryUnlinked),
            _ => Ok(state)
        }
    }

    /// Locks the state as a writer and checks to make sure this directory is still linked. If it
    /// is not this returns Err(DirectoryUnlinked).
    #[inline]
    fn checked_lock_writer(&self) -> KernResult<WriterGuard<VFSNodeState>> {
        let state = self.state.lock_writer();
        match state.parent {
            VFSParent::Unlinked => Err(DirectoryUnlinked),
            _ => Ok(state)
        }
    }
}

impl HasRc for VFSNode {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}

impl Node for VFSNode {
   
    fn list<'a>(&'a self) -> KernResult<Box<Iterator<Item=&'a str> + 'a>> {
        let state = try!(self.checked_lock_reader());
        let boxed = try!(Box::new(VFSFileIter {
            lock: state.map(|state| state.entries.iter_keys())
        }));
        Ok(boxed)
    }

    fn count(&self) -> usize {
        self.state.lock_reader().entries.count()
    }

    fn open_file(&self, file: &str) -> KernResult<Box<File>> {
        trace!("opening file '{}'", file);
        let state = self.state.lock_reader();
        match state.entries.lookup(file) {
            Some(&VFSEntry::File{ ref file, .. }) => {
                let clone = try!(file.clone());  
                let boxed = try!(Box::new(clone));
                Ok(boxed)
            }
            _ => Err(NoSuchFile)
        }
        
    }

    fn open_node(&self, node: &str) -> KernResult<Rc<Node>> {
        trace!("opening node '{}'", node);
        let state = try!(self.checked_lock_reader());
        if node == PARENT_DIR {
            // The parent directory is contained in the parent field.
            match state.parent {
                VFSParent::Linked(ref parent) => Ok(parent.clone()),
                VFSParent::Root => Err(NoSuchDirectory),
                VFSParent::Unlinked => unreachable!(),
            }
        } else {
            match state.entries.lookup(node) {
                Some(&VFSEntry::Node { ref node, .. }) => {
                    Ok(node.clone())
                }
                Some(&VFSEntry::Mount { ref fs, .. }) => {
                    fs.root_node()
                }
                _ => Err(NoSuchDirectory)
            }
        }
    }

    fn make_file(&self, name: String) -> KernResult<()> {
        trace!("making file '{}'", name);
        let mut state = try!(self.checked_lock_writer());
        if state.entries.contains(name.as_str()) {
            Err(FileExists)
        } else {
            let file = try!(VFSFile::new());
            let entry = VFSEntry::File { name: name, file: file, link: DoubleLink::new() };
            let entry = try!(Box::new(entry));
            assert!(state.entries.insert(entry).is_none());
            Ok(())
        }
    }

    fn make_node(&self, name: String) -> KernResult<()> {
        trace!("making node '{}'", name);
        let mut state = try!(self.checked_lock_writer());
        if state.entries.contains(name.as_str()) {
            trace!("failed, {} already exists", name);
            Err(DirectoryExists)
        } else {
            let parent = VFSParent::Linked(Rc::from_ref(self));
            let node = try!(VFSNode::new(parent).and_then(Box::new).map(Rc::new));
            let entry = VFSEntry::Node { name: name, node: node, link: DoubleLink::new() };
            let entry = try!(Box::new(entry));
            assert!(state.entries.insert(entry).is_none());
            Ok(())
        }
    }

    fn remove_file(&self, name: &str) -> KernResult<()> {
        trace!("removing file: '{}'", name);
        let mut state = try!(self.checked_lock_writer());
        match state.entries.remove(name) {
            None => Err(NoSuchFile),
            Some(_) => Ok(())
        }
    }

    fn remove_node(&self, name: &str) -> KernResult<()> {
        trace!("removing node: {}", name);
        let mut state = try!(self.checked_lock_writer());
        match state.entries.lookup(name) {
            None => return Err(NoSuchDirectory),
            Some(&VFSEntry::File { .. }) => return Err(NoSuchDirectory),
            Some(&VFSEntry::Node { ref node, .. }) => try!(node.unlink()),
            Some(&VFSEntry::Mount { .. }) => unimplemented!(),
        }
        state.entries.remove(name);
        Ok(())
    }

    fn mount(&self, name: String, fs: Box<FileSystem>) -> KernResult<()> {
        let mut state = try!(self.checked_lock_writer());
        if state.entries.contains(name.as_str()) {
            Err(DirectoryExists)
        } else {
            let entry = VFSEntry::Mount { name: name, fs: fs, link: DoubleLink::new() };
            let entry = try!(Box::new(entry));
            assert!(state.entries.insert(entry).is_none());
            Ok(())
        }
    }

    fn unlink(&self) -> KernResult<()> {
        let mut state = try!(self.checked_lock_writer());
        if state.entries.count() != 0 {
            Err(DirectoryNotEmpty)
        } else {
            state.parent = VFSParent::Unlinked;
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
