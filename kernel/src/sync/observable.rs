use alloc::sync::Arc;
use spin::Spin;

use crate::task::{processor::Processor, task::Task};

use super::waiting_queue::WaitingQueue;

pub struct Observable {
    waitings: Spin<WaitingQueue>,
}

impl Observable {
    pub fn new() -> Self {
        Self {
            waitings: Spin::new(WaitingQueue::new()),
        }
    }

    pub fn wait(&self, task: Arc<Task>) {
        self.waitings.lock().push(&task);
        drop(task);
        Processor::suspend();
    }

    pub fn notify_one(&self) {
        let task = self.waitings.lock().pop();
        if let Some(task) = task {
            task.wakeup();
        }
    }

    pub fn notify_all(&self) {
        let mut waitings = self.waitings.lock();
        while let Some(task) = waitings.pop() {
            task.wakeup();
        }
    }
}

impl Default for Observable {
    fn default() -> Self {
        Self::new()
    }
}
