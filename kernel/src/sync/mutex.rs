use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{sync::Weak, vec::Vec};
use spin::mutex::Mutex;

use crate::task::{manager::TASK_MANAGER, processor::fetch_curr_task, suspend_yield, task::Task};

pub struct SpinMutex {
    locked: AtomicBool, // actually, it doesn't need atomic
}

pub struct BlockMutex {
    locked: AtomicBool, // actually, it doesn't need atomic
    waitings: Mutex<Vec<Weak<Task>>>,
}

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

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }
}

impl BlockMutex {
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            let task = fetch_curr_task();
            task.stop();

            self.waitings.lock().push(task.phantom());
            suspend_yield();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);

        let task = loop {
            let task = self.waitings.lock().pop();
            if let Some(task) = task {
                if let Some(task) = task.upgrade() {
                    break Some(task);
                }
            } else {
                break None;
            }
        };
        if let Some(task) = task {
            task.wake_up();
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }
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
        Self {
            locked: AtomicBool::new(false),
            waitings: Mutex::new(Vec::new()),
        }
    }
}
