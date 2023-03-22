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

    /// A special task context that is used for thread switching.
    idle_task_ctx: TaskContext,

    /// The current task that the processor is executing.
    curr_task: Option<Arc<Mutex<Task>>>,
}

impl Manager {
    pub fn new(task: Task) -> Self {
        let mut tasks: VecDeque<Arc<Mutex<Task>>> = VecDeque::new();
        tasks.push_back(Arc::new(Mutex::new(task)));
        Self {
            tasks,
            idle_task_ctx: TaskContext::empty(),
            curr_task: None,
        }
    }

    pub fn push_task(&mut self, task: Task) {
        self.tasks.push_back(Arc::new(Mutex::new(task)));
    }

    pub fn curr_task(&self) -> Option<Arc<Mutex<Task>>> {
        self.curr_task.clone()
    }

    pub fn idle_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_ctx
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<Manager> =
        Mutex::new(Manager::new(Task::from_elf(get_app_data(0), None)));
}

pub fn fetch_curr_task() -> Arc<Mutex<Task>> {
    TASK_MANAGER
        .lock()
        .curr_task()
        .expect("[kernel] There is no running task currently.")
}

pub fn fetch_idle_task_ctx() -> *mut TaskContext {
    &mut TASK_MANAGER.lock().idle_task_ctx
}

pub fn switch() {
    let mut task_manager = TASK_MANAGER.lock();
    let task = task_manager.tasks.pop_front();
    if let Some(task) = task {
        let task_ctx = task.lock().task_ctx_ptr();
        let idle_task_ctx = task_manager.idle_task_ctx_ptr();
        task_manager.curr_task = Some(task);
        drop(task_manager);

        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe {
            _switch(idle_task_ctx, task_ctx);
        }

        // clear current task
        let mut task_manager = TASK_MANAGER.lock();
        let curr_task = task_manager.curr_task().unwrap();
        if *curr_task.lock().task_status() != TaskStatus::Zombie {
            task_manager.tasks.push_back(curr_task);
            task_manager.curr_task = None;
        }
    } else {
        panic!("[kernel] Shutdown.");
    }
}

pub fn init_tasks() {
    let num_apps = get_num_apps();
    for i in 1..num_apps {
        let task = Task::from_elf(get_app_data(i), None);
        TASK_MANAGER.lock().push_task(task);
    }
}

pub fn run_tasks() {
    loop {
        println!("[kernel] Begin to do the switching.");
        switch();
    }
}
