use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{sync::Weak, vec::Vec};
use spin::mutex::Mutex;

use crate::task::{processor::fetch_curr_task, suspend_yield, task::Task};

pub struct Semaphore {
    inner: Mutex<SemaphoreInner>,
}

pub struct SemaphoreInner {
    counter: usize,
    waitings: Vec<Weak<Task>>,
}

impl Semaphore {
    pub fn new(counter: usize) -> Semaphore {
        Self {
            inner: Mutex::new(SemaphoreInner {
                counter,
                waitings: Vec::new(),
            }),
        }
    }

    pub fn down(&self) {
        let sema = &self.inner;
        while sema.lock().counter == 0 {
            let task = fetch_curr_task();
            task.stop();

            sema.lock().waitings.push(task.phantom());
            suspend_yield();
        }
        sema.lock().counter -= 1;
    }

    pub fn up(&self) {
        let sema = &self.inner;
        if sema.lock().counter == 0 {
            let task = loop {
                let task = sema.lock().waitings.pop();
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
        sema.lock().counter += 1;
    }
}
