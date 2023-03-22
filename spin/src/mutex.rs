use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Mutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {}
        MutexGuard::new(&self.lock, unsafe { &mut *self.data.get() }) // bypass mutability check
    }
}

impl<'a, T: 'a> MutexGuard<'a, T> {
    pub fn new(lock: &'a AtomicBool, data: &'a mut T) -> Self {
        Self { lock, data }
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
        self.lock.store(false, Ordering::Release);
    }
}
