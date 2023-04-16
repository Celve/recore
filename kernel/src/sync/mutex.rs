use core::sync::atomic::{AtomicBool, Ordering};

use crate::task::suspend_yield;

pub struct SpinMutex {
    locked: AtomicBool,
}

pub struct BlockMutex {}

impl SpinMutex {
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            suspend_yield();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

impl BlockMutex {
    pub fn lock(&self) {}

    pub fn unlock(&self) {}
}

impl SpinMutex {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}

impl BlockMutex {
    pub fn new() -> Self {
        todo!()
    }
}
