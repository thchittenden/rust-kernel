use core::any::Any;
use collections::queue::Queue;
use mutex::{Mutex, MutexGuard};
use task::thread::Thread;

pub struct CondVar {
    pub queue: Mutex<Queue<Thread>>
}

macro_rules! static_condvar {
    () => ({
        // I don't know why we can't use the static_queue!() macro here...
        use collections::queue::Queue;
        CondVar {
            queue: static_mutex!(Queue { head: None, tail: None })
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
