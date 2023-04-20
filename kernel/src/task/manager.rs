use core::num::NonZeroUsize;

use super::task::Task;

use alloc::sync::{Arc, Weak};
use lazy_static::lazy_static;
use lru::LruCache;
use spin::Spin;

pub struct TaskManager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    tasks: Spin<LruCache<(usize, usize), Weak<Task>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Spin::new(LruCache::new(NonZeroUsize::new(1024).unwrap())),
        }
    }

    /// Push a new task to the task manager.
    pub fn push(&self, task: &Arc<Task>) {
        let pid = task.proc().pid();
        let tid = task.lock().tid();
        assert!(self.tasks.lock().get(&(pid, tid)).is_none());
        self.tasks.lock().push((pid, tid), task.phantom());
    }

    /// Pop the least recently executed task.
    pub fn pop(&self) -> Option<Arc<Task>> {
        self.tasks
            .lock()
            .pop_lru()
            .and_then(|(_, task)| task.upgrade())
    }

    pub fn remove(&self, pid: usize, tid: usize) {
        self.tasks.lock().pop(&(pid, tid));
    }

    pub fn len(&self) -> usize {
        self.tasks.lock().len()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = TaskManager::new();
}
