use alloc::sync::Arc;
use spin::mutex::Mutex;

use crate::task::task::Task;

use super::waiting_queue::WaitingQueue;

pub struct Condvar {
    waitings: Mutex<WaitingQueue>,
}

impl Condvar {
    pub fn new() -> Self {
        Self {
            waitings: Mutex::new(WaitingQueue::new()),
        }
    }

    pub fn wait(&self, task: &Arc<Task>) {
        self.waitings.lock().push(&task);
        task.suspend();
    }

    pub fn notify_one(&self) {
        let task = self.waitings.lock().pop();
        if let Some(task) = task {
            task.wake_up();
        }
    }

    pub fn notify_all(&self) {
        let mut waitings = self.waitings.lock();
        while let Some(task) = waitings.pop() {
            task.wake_up();
        }
    }
}

impl Default for Condvar {
    fn default() -> Self {
        Self::new()
    }
}
