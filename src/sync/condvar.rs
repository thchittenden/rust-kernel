use core::any::Any;
use collections::linkedlist::LinkedList;
use mutex::{Mutex, MutexGuard};
use task::thread::Thread;

pub struct CondVar {
    pub linkedlist: Mutex<LinkedList<Thread>>
}

macro_rules! static_condvar {
    () => ({
        // I don't know why we can't use the static_linkedlist!() macro here...
        use collections::linkedlist::LinkedList;
        CondVar {
            linkedlist: static_mutex!(LinkedList { head: None, tail: None })
        }
    });
}

impl CondVar {
    
    pub fn wait<T>(&self, lock: &MutexGuard<T>) {
        unimplemented!()
    }

    pub fn signal(&self) {
        unimplemented!()
    }

}
