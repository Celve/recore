#![no_std]

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Default, Debug)]
pub struct Spin<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinGuard<'a, T: 'a> {
    lock: &'a AtomicBool,
    spin: &'a Spin<T>,
    data: &'a mut T,
}

impl<T> Spin<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}

unsafe impl<T: Send> Sync for Spin<T> {}
unsafe impl<T: Send> Send for Spin<T> {}

impl<T> Spin<T> {
    pub fn lock(&self) -> SpinGuard<T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {}
        SpinGuard::new(&self, unsafe { &mut *self.data.get() }) // bypass mutability check
    }

    pub fn try_lock(&self) -> Option<SpinGuard<T>> {
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            Some(SpinGuard::new(&self, unsafe { &mut *self.data.get() }))
        } else {
            None
        }
    }
}

impl<'a, T: 'a> SpinGuard<'a, T> {
    pub fn new(spin: &'a Spin<T>, data: &'a mut T) -> Self {
        Self {
            lock: &spin.lock,
            spin,
            data,
        }
    }

    pub fn spin(&self) -> &'a Spin<T> {
        self.spin
    }
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
