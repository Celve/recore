use crate::task::processor::fetch_curr_task;

use super::{mutex::MutexGuard, observable::Observable};

pub struct Condvar {
    inner: Observable,
}

impl Condvar {
    pub fn wait<'a, T>(&'a self, guard: MutexGuard<'a, T>) -> MutexGuard<T> {
        let mutex = guard.mutex();
        drop(guard);
        self.inner.wait(&fetch_curr_task());
        mutex.lock()
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
