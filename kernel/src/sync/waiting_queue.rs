use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::task::task::Task;

pub struct WaitingQueue {
    waitings: Vec<Weak<Task>>,
}

impl WaitingQueue {
    pub const fn new() -> Self {
        Self {
            waitings: Vec::new(),
        }
    }

    pub fn push(&mut self, task: &Arc<Task>) {
        self.waitings.push(task.phantom());
    }

    pub fn pop(&mut self) -> Option<Arc<Task>> {
        loop {
            let opt_task = self.waitings.pop();
            if let Some(weak_task) = opt_task {
                if let Some(arc_task) = weak_task.upgrade() {
                    break Some(arc_task);
                }
            } else {
                break None;
            }
        }
    }
}
