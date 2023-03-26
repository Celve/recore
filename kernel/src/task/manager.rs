use super::task::Task;

use crate::task::loader::get_app_data;

use alloc::{collections::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use spin::mutex::Mutex;

pub struct Manager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    tasks: VecDeque<Arc<Task>>,
}

impl Manager {
    pub fn new(task: Arc<Task>) -> Self {
        let mut tasks: VecDeque<Arc<Task>> = VecDeque::new();
        tasks.push_back(task);
        Self { tasks }
    }

    pub fn push(&mut self, task: Arc<Task>) {
        self.tasks.push_back(task);
    }

    pub fn pop(&mut self) -> Option<Arc<Task>> {
        self.tasks.pop_front()
    }
}

lazy_static! {
    pub static ref INITPROC: Arc<Task> = Arc::new(Task::from_elf(
        get_app_data("initproc").expect("[task] Unable to load initproc."),
        None
    ));

    /// Manager only loads the initproc at the beginning.
    pub static ref MANAGER: Mutex<Manager> = Mutex::new(Manager::new(INITPROC.clone()));
}
