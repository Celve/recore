use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use super::basic::BlockLock;

pub struct Mutex<T> {
    lock: BlockLock,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a BlockLock,
    mutex: &'a Mutex<T>,
    data: &'a mut T,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: BlockLock::new(),
            data: UnsafeCell::new(data),
        }
    }
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<T> {
        self.lock.lock();
        MutexGuard::new(&self, unsafe { &mut *self.data.get() }) // bypass mutability check
    }

    // pub fn try_lock(&self) -> Option<MutexGuard<T>> {}
}

impl<'a, T: 'a> MutexGuard<'a, T> {
    pub fn new(mutex: &'a Mutex<T>, data: &'a mut T) -> Self {
        Self {
            lock: &mutex.lock,
            mutex,
            data,
        }
    }

    pub fn mutex(&self) -> &'a Mutex<T> {
        self.mutex
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
