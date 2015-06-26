use core::prelude::*;
use core::atomic::{AtomicBool, Ordering};
use core::mem;
use collections::link::{DoubleLink, HasDoubleLink};
use collections::dlist::DList;
use mutex::{Mutex, MutexGuard};
use task::thread::Thread;

use alloc::boxed::Box; // This should definitely not be here.

#[allow(improper_ctypes)]
extern {
    fn sched_yield(tid: Option<usize>);
}

struct CondVarNode {
    signaled: &'static AtomicBool,
    link: DoubleLink<CondVarNode>,
}   

impl CondVarNode {
    fn new(signaled: &'static AtomicBool) -> CondVarNode {
        CondVarNode {
            signaled: signaled,
            link: DoubleLink::new(),
        }
    }
}

impl HasDoubleLink<CondVarNode> for CondVarNode {
    fn dlink(&self) -> &DoubleLink<CondVarNode> {
        &self.link
    }
    fn dlink_mut(&mut self) -> &mut DoubleLink<CondVarNode> {
        &mut self.link
    }
}

pub struct CondVar {
    list: Mutex<DList<CondVarNode>>
}

impl CondVar {

    pub const fn new() -> CondVar {
        CondVar {
            list: Mutex::new(DList::new())
        }
    }
   
    /// Blocks the calling thread until another thread signals it. 
    ///
    /// Currently this is implemented very naively and unsafely. Once descheduling queues are
    /// created however this should use those. TODO.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let old_guard = guard.unlock();
        let signal = AtomicBool::new(false);
        let node = CondVarNode::new(unsafe { mem::transmute(&signal) });
        let bnode = Box::new(node).unwrap();
        self.list.lock().push_tail(bnode);

        while !signal.load(Ordering::Relaxed) {
            unsafe { sched_yield(None) };
        }

        old_guard.relock()
    }

    pub fn signal(&self) {
        unimplemented!()
    }

    pub fn broadcast(&self) {
        unimplemented!()
    }

}
