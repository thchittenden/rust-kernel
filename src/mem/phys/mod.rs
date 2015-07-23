mod frame;

pub use self::frame::Frame;
use self::frame::FreeFrame;
use self::FreeFrameListHead::*;
use core::prelude::*;
use core::mem;
use core::atomic::{AtomicUsize, Ordering};
use mutex::Mutex;
use util::{PAGE_SIZE, is_page_aligned};
use util::KernResult;
use util::KernError::*;
use virt::AddressSpace;
use virt::pt::{PTE_SUPERVISOR, PTE_WRITABLE};
use virt::page::Page;
logger_init!(Trace);

static FREE_FRAME_LIST: Mutex<FreeFrameList> = Mutex::new(FreeFrameList::new());

/// The free frame list.
struct FreeFrameList {
    head: FreeFrameListHead,
    max: usize,
    count: usize,
    reserved: usize,
}

enum FreeFrameListHead {
    NotPaging(Option<Frame<FreeFrame>>),
    Paging(&'static AddressSpace),
}

impl FreeFrameListHead {
   
    fn paging_enabled(&self) -> bool {
        match self {
            &NotPaging(_) => false,
            &Paging(_) => true,
        }
    }

    fn push(&mut self, frame: Frame<()>) {
        match self {
            &mut NotPaging(ref mut head) =>  {
                let mut frame = frame.allocate(FreeFrame::new());
                frame.next = head.take();
                *head = Some(frame);
            }
            &mut Paging(addrspace) => {
                let addr = linker_sym!(__ffl_head);
                let mut lock = addrspace.lock_page_writer(addr);
                unsafe {
                    // Unmap the old top of the FFL.
                    let oldtop = lock.unmap_page(addr);

                    // Map in the new frame as the new head.
                    let flags = PTE_SUPERVISOR | PTE_WRITABLE;
                    lock.map_page(addr, frame, flags);

                    // Create a reference to the new top of the list.
                    let mut top: Page<FreeFrame> = Page::from_addr(addr);

                    // Set the head to point to the old top.
                    top.next = Some(oldtop.cast::<FreeFrame>()); 
                }
            }
        }
    }

    fn pop(&mut self) -> Option<Frame<()>> {
        match self {
            &mut NotPaging(ref mut head) => {
                head.take().map(|mut top| {
                    *head = top.next.take();
                    top.unallocate()
                })
            }
            &mut Paging(addrspace) => {
                let addr = linker_sym!(__ffl_head);
                let mut lock = addrspace.lock_page_writer(addr);
                unsafe { 
                    if lock.has_page(addr) {
                        // Create a reference to the top of the list.
                        let mut top: Page<FreeFrame> = Page::from_addr(addr);
                       
                        // Get the next top of the list.
                        let next = top.next.take();

                        // Unmap the old frame.
                        let oldtop = lock.unmap_page(addr);

                        // If the next frame was not empty, map it in.
                        if let Some(next) = next {
                            let flags = PTE_SUPERVISOR | PTE_WRITABLE;
                            lock.map_page(addr, next.cast::<()>(), flags);
                        }   

                        // Return the old top frame.
                        Some(oldtop)
                    } else {
                        None
                    }
                }
            }
        }
    }
}

impl FreeFrameList {
    const fn new() -> FreeFrameList {
        FreeFrameList {
            head: NotPaging(None),
            max: 0,
            count: 0,
            reserved: 0,
        }
    }

    fn initialize(&mut self, lo: usize, hi: usize) {
        assert!(!self.head.paging_enabled());
        assert!(is_page_aligned(lo));
        assert!(is_page_aligned(hi));
        for addr in (lo..hi).step_by(PAGE_SIZE) {
            // Filter out the zero frame because a frame with address 0 is "not present". This
            // check probably does not belong here.
            if addr == 0 { continue }
            
            // Construct the frame and adds it to the free list. We know this is safe because we
            // assume the user calls initialize once for a given range.
            let frame = unsafe { Frame::<()>::from_addr(addr) };
            self.head.push(frame);
            self.max += 1;
            self.count += 1;
        }
    }

    fn enable_paging(&mut self, addrspace: &'static AddressSpace) {
        trace!("enabling phys paging path");
        assert!(!self.head.paging_enabled());
        self.head = match &mut self.head {
            &mut NotPaging(ref mut top) => {
                // Create a new free frame pointing to the old top of the list at the FFL head entry.
                // Forget it so that it doesn't try to be freed.
                let addr = linker_sym!(__ffl_head);
                unsafe { Page::<()>::from_addr(addr) }.allocate(FreeFrame { next: top.take() });

                // Update the head to be in Paging mode.
                Paging(addrspace)
            }
            _ => unreachable!()
        };
    }

    fn reserve(&mut self, count: usize) -> KernResult<()> {
        if self.reserved + count <= self.count {
            self.reserved += count;
            Ok(())
        } else {
            Err(OutOfMemory)
        }
    }

    fn unreserve(&mut self, count: usize) {
        assert!(self.reserved >= count);
        self.reserved -= count;
    }

    fn get_unreserved(&mut self) -> KernResult<Frame<()>> {
        if self.count > self.reserved {
            self.count -= 1;
            Ok(self.head.pop().unwrap())
        } else {
            Err(OutOfMemory)
        }
    }


    fn get_reserved(&mut self) -> Frame<()> {
        assert!(self.reserved > 0);
        self.count -= 1;
        self.reserved -= 1;
        self.head.pop().unwrap()
    }

    fn return_frame(&mut self, frame: Frame<()>) {
        self.head.push(frame);
        self.count += 1;
    }
}

/// A struct representing how many frames an address space has reserved. It must always be possible
/// to retrieve this many frames from the free frame list.
pub struct FrameReserve {
    count: AtomicUsize,
}

impl FrameReserve {
    
    pub fn new() -> FrameReserve {
        FrameReserve { count: AtomicUsize::new(0) }
    }

    pub fn reserve(&self, count: usize) -> KernResult<()> {
        try!(FREE_FRAME_LIST.lock().reserve(count));
        self.count.fetch_add(count, Ordering::Relaxed);
        Ok(())
    }

    pub fn unreserve(&self, count: usize) {
        FREE_FRAME_LIST.lock().unreserve(count)
    }

    pub fn split(&self, new_count: usize) -> FrameReserve {
        self.count.fetch_sub(new_count, Ordering::Relaxed);
        FrameReserve { count: AtomicUsize::new(new_count) }
    }

    pub fn get_frame(&self) -> Frame<()> {
        assert!(self.count.fetch_sub(1, Ordering::Relaxed) > 0);
        FREE_FRAME_LIST.lock().get_reserved()
    }

    pub fn get_frame_unreserved(&self) -> KernResult<Frame<()>> {
        FREE_FRAME_LIST.lock().get_unreserved()
    }
}

impl Drop for FrameReserve {
    fn drop(&mut self) {
        let count = self.count.swap(0, Ordering::Relaxed);
        FREE_FRAME_LIST.lock().unreserve(count);
    }   
}

/// Adds a range of physical memory to the free frame list. This assumes that these ranges do not
/// overlap any ranges already added to the free frame list.
pub fn add_range(start: usize, end: usize) {
    trace!("adding range: {:x}-{:x}", start, end);
    FREE_FRAME_LIST.lock().initialize(start, end);
}

/// Enables paging in the phys module. 
pub fn enable_paging(addrspace: &'static AddressSpace) {
    FREE_FRAME_LIST.lock().enable_paging(addrspace)
}

pub fn init() {

}
