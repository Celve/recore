use core::arch::asm;

use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::task::task::TaskStatus;

use super::{
    loader::{get_app_data, get_num_apps},
    manager::MANAGER,
    task::{Task, TaskContext},
};

pub struct Processor {
    /// The current task that the processor is exeucting.
    curr_task: Option<Arc<Mutex<Task>>>,

    /// A special task context that is used for thread switching.
    idle_task_ctx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            curr_task: None,
            idle_task_ctx: TaskContext::empty(),
        }
    }

    pub fn idle_task_ctx(&self) -> &TaskContext {
        &self.idle_task_ctx
    }

    pub fn idle_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_ctx
    }

    pub fn clone_curr_task(&self) -> Option<Arc<Mutex<Task>>> {
        self.curr_task.clone()
    }

    pub fn take_curr_task(&mut self) -> Option<Arc<Mutex<Task>>> {
        self.curr_task.take()
    }

    pub fn insert_curr_task(&mut self, task: Arc<Mutex<Task>>) {
        self.curr_task = Some(task);
    }
}

lazy_static! {
    static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

pub fn fetch_curr_task() -> Arc<Mutex<Task>> {
    PROCESSOR
        .lock()
        .clone_curr_task()
        .expect("[kernel] There is no running task currently.")
}

pub fn fetch_idle_task_ctx() -> *mut TaskContext {
    PROCESSOR.lock().idle_task_ctx_ptr()
}

pub fn switch() {
    let task = MANAGER.lock().pop();
    if let Some(task) = task {
        let task_ctx = task.lock().task_ctx_ptr();
        let idle_task_ctx = PROCESSOR.lock().idle_task_ctx_ptr();
        PROCESSOR.lock().insert_curr_task(task);

        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe {
            _switch(idle_task_ctx, task_ctx);
        }

        // clear current task
        let curr_task = PROCESSOR.lock().take_curr_task().unwrap();
        if *curr_task.lock().task_status() != TaskStatus::Zombie {
            MANAGER.lock().push(curr_task);
        } else {
            println!("[kernel] One process has ended.");
        }
    } else {
        panic!("[kernel] Shutdown.");
    }
}

pub fn init_tasks() {
    let num_apps = get_num_apps();
    for i in 1..num_apps {
        let task = Task::from_elf(get_app_data(i), None);
        MANAGER.lock().push(Arc::new(Mutex::new(task)));
    }
}

pub fn run_tasks() {
    loop {
        println!("[kernel] Begin to do the switching.");
        switch();
    }
}
