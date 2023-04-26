use core::sync::atomic::{AtomicBool, Ordering};

use spin::Spin;

use crate::task::processor::Processor;

use super::waiting_queue::WaitingQueue;

pub struct SpinLock {
    lock: AtomicBool, // actually, it doesn't need atomic
}

pub struct BlockLock {
    lock: AtomicBool, // actually, it doesn't need atomic
    queue: Spin<WaitingQueue>,
}

impl SpinLock {
    pub fn lock(&self) {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            Processor::yield_now();
        }
    }

    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Acquire)
    }
}

impl BlockLock {
    pub fn lock(&self) {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            {
                let task = Processor::curr_task();
                self.queue.lock().push(&task);
            }
            Processor::suspend();
        }
    }

    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release);

        let task = self.queue.lock().pop();
        if let Some(task) = task {
            task.wakeup();
        }
    }

    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Acquire)
    }
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
        }
    }
}

impl BlockLock {
    pub const fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
            queue: Spin::new(WaitingQueue::new()),
        }
    }
}
