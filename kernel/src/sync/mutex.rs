use core::sync::atomic::{AtomicBool, Ordering};

use spin::mutex::Mutex;

use crate::task::processor::fetch_curr_task;

use super::waiting_queue::WaitingQueue;

pub struct SpinMutex {
    locked: AtomicBool, // actually, it doesn't need atomic
}

pub struct BlockMutex {
    locked: AtomicBool, // actually, it doesn't need atomic
    queue: Mutex<WaitingQueue>,
}

impl SpinMutex {
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            fetch_curr_task().yield_now();
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
            self.queue.lock().push(&task);
            task.suspend();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);

        let task = self.queue.lock().pop();
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
            queue: Mutex::new(WaitingQueue::new()),
        }
    }
}
