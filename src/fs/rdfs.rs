use alloc::boxed::Box;
use alloc::rc::{Rc, HasRc, RcAny};
use collections::link::{SingleLink, HasSingleLink};
use collections::hashmap::{HashMap, HasKey, KeyIter};
use collections::string::String;
use core::prelude::*;
use core::atomic::AtomicUsize;
use core::slice::from_raw_parts;
use core::str::from_utf8;
use sync::rwlock::RWLock;
use super::{Node, File, FileSystem};
use util::KernResult;
use util::KernError::*;
use super::PARENT_DIR;
logger_init!(Trace);

/// A file system interface to the RamDisk.
pub struct RDFS {
    root: Rc<RDFSNode>
}

impl RDFS {
    pub fn new() -> KernResult<RDFS> {
        let node = try!(RDFSNode::new(None).and_then(Box::new));
        let root = Rc::new(node);
        Ok(RDFS { root: root })
    }
}

impl FileSystem for RDFS {
    fn root_node(&self) -> KernResult<Rc<Node>> {
        Ok(self.root.clone())
    }
    fn set_parent(&mut self, parent: Option<Rc<Node>>) {
        let mut root_parent = self.root.parent.lock_writer();
        *&mut *root_parent = parent;
    }
}

struct RDFSDirEntry {
    link: SingleLink<RDFSDirEntry>,
    name: &'static str,
    size: usize,
}

impl RDFSDirEntry {
    fn new(name: &'static str, size: usize) -> RDFSDirEntry {
        RDFSDirEntry {
            link: SingleLink::new(),
            name: name,
            size: size
        }
    }
}

impl HasSingleLink<RDFSDirEntry> for RDFSDirEntry {
    fn slink(&self) -> &SingleLink<RDFSDirEntry> {
        &self.link
    }
    fn slink_mut(&mut self) -> &mut SingleLink<RDFSDirEntry> {
        &mut self.link
    }
}

impl HasKey<str> for RDFSDirEntry {
    fn get_key(&self) -> &str {
        self.name
    }
}

struct RDFSNode {
    rc: AtomicUsize,
    parent: RWLock<Option<Rc<Node>>>,
    directory: HashMap<str, RDFSDirEntry>,
    ramdisk: &'static [u8],
}

fn make_ramdisk() -> &'static [u8] {
    let start = linker_sym!(__ramdisk_start);
    let size = linker_sym!(__ramdisk_size);
    unsafe { from_raw_parts(start as *const u8, size) }
}
fn make_directory() -> KernResult<HashMap<str, RDFSDirEntry>> {
    trace!("scanning ramdisk");
    let start = linker_sym!(__ramdisk_dir_start);
    let size = linker_sym!(__ramdisk_dir_size);
    let mut map = try!(HashMap::new());
    let slice = unsafe { from_raw_parts(start as *const u8, size) };
    let mut slice_iter = slice.split(|&c| c == 0);
    loop {
        let name_raw = slice_iter.next().unwrap();
        if name_raw.len() == 0 {
            // A 0-length name indicates the end of the directory.
            break;
        }
        let addr_raw = slice_iter.next().unwrap();
        let name = from_utf8(name_raw).unwrap();
        let addr = from_utf8(addr_raw).unwrap();
        let size = usize::from_str_radix(addr, 16).unwrap();
        trace!("found file: {}, {}", name, addr);

        // Insert the entry into the directory.
        let entry = try!(Box::new(RDFSDirEntry::new(name, size)));
        map.insert(entry);
    }
    Ok(map)
}

impl RDFSNode {
    fn new(parent: Option<Rc<Node>>) -> KernResult<RDFSNode> {
        Ok(RDFSNode {
            rc: AtomicUsize::new(0),
            parent: RWLock::new(parent),
            directory: try!(make_directory()),
            ramdisk: make_ramdisk(),
        })
    }
}

impl Node for RDFSNode {

    fn count(&self) -> usize {
        self.directory.count()
    }
    
    fn list<'a>(&'a self) -> KernResult<Box<Iterator<Item=&'a str> + 'a>> {
        Ok(try!(Box::new(self.directory.iter_keys())))
    }

    fn make_file(&self, file: String) -> KernResult<()> {
        Err(Unsupported)
    }
    
    fn make_node(&self, node: String) -> KernResult<()> {
        Err(Unsupported)
    }

    fn make_object(&self, name: String, obj: Rc<RcAny>) -> KernResult<()> {
        Err(Unsupported)
    }

    fn open_file(&self, node: &str) -> KernResult<Box<File>> {
        unimplemented!()
    }

    fn open_node(&self, node: &str) -> KernResult<Rc<Node>> {
        if node == PARENT_DIR {
            match *self.parent.lock_reader() {
                None => Err(NoSuchDirectory),
                Some(ref parent) => Ok(parent.clone())
            }
        } else {
            Err(NoSuchDirectory)
        }
    }

    fn open_object(&self, node: &str) -> KernResult<Rc<RcAny>> {
        Err(NoSuchObject)
    }

    fn remove_file(&self, file: &str) -> KernResult<()> {
        Err(Unsupported)
    }

    fn remove_node(&self, node: &str) -> KernResult<()> {
        Err(Unsupported)
    }

    fn remove_object(&self, name: &str) -> KernResult<Rc<RcAny>> {
        Err(Unsupported)
    }

    fn mount(&self, node: String, fs: Box<FileSystem>) -> KernResult<()> {
        Err(Unsupported)
    }

    fn unlink(&self) -> KernResult<()> {
        Ok(())
    }

}

impl HasRc for RDFSNode {
    fn get_count(&self) -> &AtomicUsize {
        &self.rc
    }
}
