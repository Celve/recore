use spin::SpinGuard;

use crate::task::processor::Processor;

use super::{mutex::MutexGuard, observable::Observable};

pub struct Condvar {
    inner: Observable,
}

impl Condvar {
    pub fn wait_mutex<'a, T>(&'a self, guard: MutexGuard<'a, T>) -> MutexGuard<T> {
        let lock = guard.mutex();
        drop(guard);
        self.inner.wait(&Processor::curr_task());
        lock.lock()
    }

    pub fn wait_spin<'a, T>(&'a self, guard: SpinGuard<'a, T>) -> SpinGuard<T> {
        let lock = guard.spin();
        drop(guard);
        self.inner.wait(&Processor::curr_task());
        lock.lock()
    }

    pub fn notify_one(&self) {
        self.inner.notify_one();
    }

    pub fn notify_all(&self) {
        self.inner.notify_all();
    }
}

impl Condvar {
    pub fn new() -> Self {
        Self {
            inner: Observable::new(),
        }
    }
}
