use core::num::NonZeroUsize;

use super::task::Task;

use alloc::sync::Arc;
use lazy_static::lazy_static;
use lru::LruCache;
use spin::mutex::Mutex;

pub struct TaskManager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    tasks: Mutex<LruCache<(usize, usize), Arc<Task>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap())),
        }
    }

    /// Push a new task to the task manager.
    pub fn push(&self, task: Arc<Task>) {
        let pid = task.proc().pid();
        let tid = task.lock().tid();
        self.tasks.lock().push((pid, tid), task);
    }

    /// Pop the least recently executed task.
    pub fn pop(&self) -> Option<Arc<Task>> {
        self.tasks.lock().pop_lru().map(|(_, task)| task)
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
