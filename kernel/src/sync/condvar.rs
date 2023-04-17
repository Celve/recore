use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::mutex::Mutex;

use crate::task::{suspend_yield, task::Task};

pub struct Condvar {
    waitings: Mutex<Vec<Weak<Task>>>,
}

impl Condvar {
    pub fn new() -> Self {
        Self {
            waitings: Mutex::new(Vec::new()),
        }
    }

    pub fn wait(&self, task: &Arc<Task>) {
        task.stop();
        self.waitings.lock().push(task.phantom());
        suspend_yield();
    }

    pub fn notify_one(&self) {
        let task = loop {
            let opt_task = self.waitings.lock().pop();
            if let Some(weak_task) = opt_task {
                if let Some(arc_task) = weak_task.upgrade() {
                    break Some(arc_task);
                }
            } else {
                break None;
            }
        };
        if let Some(task) = task {
            task.wake_up();
        }
    }

    pub fn notify_all(&self) {
        let mut waitings = self.waitings.lock();
        while let Some(task) = waitings.pop() {
            if let Some(task) = task.upgrade() {
                task.wake_up();
            }
        }
    }
}
