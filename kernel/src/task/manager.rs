use core::borrow::Borrow;

use super::task::Task;

use crate::{
    println,
    sync::up::UpCell,
    task::{loader::get_app_data, task::TaskContext},
};

use alloc::{collections::VecDeque, sync::Arc, sync::Weak};
use lazy_static::lazy_static;

pub struct Manager {
    /// The first task in the task deque is the next task, while the last task in the task deque is the current task.
    tasks: VecDeque<Arc<UpCell<Task>>>,

    /// A special task context that is used for thread switching.
    idle_task_ctx: TaskContext,
}

impl Manager {
    pub fn new(task: Task) -> Self {
        let mut tasks: VecDeque<Arc<UpCell<Task>>> = VecDeque::new();
        tasks.push_back(Arc::new(UpCell::new(task)));
        Self {
            tasks,
            idle_task_ctx: TaskContext::empty(),
        }
    }

    pub fn current_task(&self) -> Weak<UpCell<Task>> {
        Arc::downgrade(
            self.tasks
                .front()
                .expect("[task_manager] Cannot fetch task from manager."),
        )
    }

    pub fn idle_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_ctx
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UpCell<Manager> =
        UpCell::new(Manager::new(Task::from_elf(get_app_data(0), None)));
}

pub fn fetch_curr_task() -> Arc<UpCell<Task>> {
    TASK_MANAGER.borrow_mut().current_task().upgrade().unwrap()
}

pub fn fetch_idle_task_ctx() -> *mut TaskContext {
    &mut TASK_MANAGER.borrow_mut().idle_task_ctx
}

pub fn switch() {
    let mut task_manager = TASK_MANAGER.borrow_mut();
    let task = task_manager.tasks.pop_front();
    if let Some(task) = task {
        let task_ctx = task.borrow_mut().task_ctx_ptr();
        let idle_task_ctx = task_manager.idle_task_ctx_ptr();
        task_manager.tasks.push_back(task);
        drop(task_manager);

        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe {
            _switch(idle_task_ctx, task_ctx);
        }
    } else {
        panic!("[kernel] There is no running task.");
    }
}

pub fn run_tasks() {
    loop {
        println!("[kernel] Begin to do the switching.");
        switch();
    }
}
