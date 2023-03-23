use super::{
    loader::get_num_apps,
    task::{Task, TaskStatus},
};

use crate::task::{loader::get_app_data, task::TaskContext};

use alloc::{collections::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use spin::mutex::Mutex;

pub struct Manager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    tasks: VecDeque<Arc<Mutex<Task>>>,
}

impl Manager {
    pub fn new(task: Arc<Mutex<Task>>) -> Self {
        let mut tasks: VecDeque<Arc<Mutex<Task>>> = VecDeque::new();
        tasks.push_back(task);
        Self { tasks }
    }

    pub fn push(&mut self, task: Arc<Mutex<Task>>) {
        self.tasks.push_back(task);
    }

    pub fn pop(&mut self) -> Option<Arc<Mutex<Task>>> {
        self.tasks.pop_front()
    }
}

lazy_static! {
    pub static ref MANAGER: Mutex<Manager> = Mutex::new(Manager::new(Arc::new(Mutex::new(
        Task::from_elf(get_app_data(0), None)
    ))));
}
