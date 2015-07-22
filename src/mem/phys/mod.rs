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
use virt::{PageDirectory, PTE_SUPERVISOR, PTE_WRITABLE};
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
    Paging(Frame<PageDirectory>), // It would be cool to implement this with an AddressWriter.
}

impl FreeFrameListHead {
   
    fn paging_enabled(&self) -> bool {
        match self {
            &NotPaging(_) => false,
            &Paging(_) => true,
        }
    }

    fn push(&mut self, mut frame: Frame<FreeFrame>) {
        match self {
            &mut NotPaging(ref mut head) =>  {
                frame.next = head.take();
                *head = Some(frame);
            }
            &mut Paging(ref mut pd) => {
                unsafe {
                    // Unmap the old top of the FFL.
                    let oldtop = pd.unmap_page(linker_sym!(__ffl_head));

                    // Map in the new frame as the new head.
                    let flags = PTE_SUPERVISOR | PTE_WRITABLE;
                    pd.map_page(linker_sym!(__ffl_head), frame.unallocate(), flags);

                    // Create a reference to the new top of the list.
                    let mut top: Frame<FreeFrame> = Frame::from_addr(linker_sym!(__ffl_head));

                    // Set the head to point to the old top.
                    top.next = Some(oldtop.allocate_raw::<FreeFrame>()); 

                    // Forget the frame we created so it's not dropped.
                    mem::forget(top);
                }
            }
        }
    }

    fn pop(&mut self) -> Option<Frame<FreeFrame>> {
        match self {
            &mut NotPaging(ref mut head) => {
                head.take().map(|mut top| {
                    *head = top.next.take();
                    top
                })
            }
            &mut Paging(ref mut pd) => {
                unsafe { 
                    if pd.has_page(linker_sym!(__ffl_head)) {
                        // Create a reference to the top of the list.
                        let mut top: Frame<FreeFrame> = Frame::from_addr(linker_sym!(__ffl_head));
                        
                        // Get the next top of the list and map it in.
                        if let Some(next) = top.next.take() {
                            let flags = PTE_SUPERVISOR | PTE_WRITABLE;
                            pd.map_page(linker_sym!(__ffl_head), next.unallocate(), flags);
                        }
                       
                        // Return the top.
                        Some(top)
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
            let frame = unsafe { Frame::<()>::from_addr(addr) }.allocate(FreeFrame::new());
            self.head.push(frame);
            self.max += 1;
            self.count += 1;
        }
    }

    fn paging_enabled(&mut self, pd: Frame<PageDirectory>) {
        trace!("enabling phys paging path");
        assert!(!self.head.paging_enabled());
        self.head = match &mut self.head {
            &mut NotPaging(ref mut top) => {
                // Create a new free frame pointing to the old top of the list at the FFL head entry.
                // Forget it so that it doesn't try to be freed.
                let addr = linker_sym!(__ffl_head);
                let frame = unsafe { Frame::<()>::from_addr(addr) }.allocate(FreeFrame { next: top.take() });
                mem::forget(frame);

                // Update the head to be in Paging mode.
                Paging(pd)
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
            Ok(self.head.pop().unwrap().unallocate())
        } else {
            Err(OutOfMemory)
        }
    }


    fn get_reserved(&mut self) -> Frame<()> {
        assert!(self.reserved > 0);
        self.count -= 1;
        self.reserved -= 1;
        self.head.pop().unwrap().unallocate()
    }

    fn return_frame(&mut self, frame: Frame<FreeFrame>) {
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
pub fn enable_paging(pd: Frame<PageDirectory>) {
    FREE_FRAME_LIST.lock().paging_enabled(pd)
}

pub fn init() {

}
