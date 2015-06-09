use collections::dlist::DList;
use mutex::{Mutex, MutexGuard};
use task::thread::Thread;

pub struct CondVar {
    pub list: Mutex<DList<Thread>>
}

macro_rules! static_condvar {
    () => ({
        // I don't know why we can't use the static_dlist!() macro here...
        use collections::dlist::DList;
        CondVar {
            list: static_mutex!(DList { len: 0, head: None, tail: None })
        }
    });
}

impl CondVar {
    
    pub fn wait<T>(&self, _: &MutexGuard<T>) {
        unimplemented!()
    }

    pub fn signal(&self) {
        unimplemented!()
    }

}
