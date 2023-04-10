use alloc::sync::Arc;
use fosix::signal::SignalFlags;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::task::{manager::INITPROC, task::TaskStatus};

use super::{
    manager::MANAGER,
    task::{Task, TaskContext},
};

pub struct Processor {
    /// The current task that the processor is exeucting.
    curr_task: Option<Arc<Task>>,

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

    pub fn idle_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_ctx
    }

    pub fn clone_curr_task(&self) -> Option<Arc<Task>> {
        self.curr_task.clone()
    }

    pub fn take_curr_task(&mut self) -> Option<Arc<Task>> {
        self.curr_task.take()
    }

    pub fn insert_curr_task(&mut self, task: Arc<Task>) {
        self.curr_task = Some(task);
    }
}

lazy_static! {
    static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

/// Fetch the current task with processor locked.
pub fn fetch_curr_task() -> Arc<Task> {
    PROCESSOR
        .lock()
        .clone_curr_task()
        .expect("[kernel] There is no running task currently.")
}

/// Fetch the idle task context pointer with processor locked.
pub fn fetch_idle_task_ctx_ptr() -> *mut TaskContext {
    PROCESSOR.lock().idle_task_ctx_ptr()
}

/// Switch from idle task to the next task.
///
/// When the next task yields, it will get into this function again.
pub fn switch() {
    let task = MANAGER.lock().pop();
    if let Some(task) = task {
        if task.lock().task_status() == TaskStatus::Ready {
            *task.lock().task_status_mut() = TaskStatus::Running;
        }

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
        if curr_task.lock().task_status() != TaskStatus::Zombie {
            MANAGER.lock().push(curr_task);
        } else {
            for task in curr_task.lock().children().iter() {
                *task.lock().parent_mut() = Some(Arc::downgrade(&INITPROC));
                INITPROC.lock().children_mut().push(task.clone());
            }

            let parent = curr_task.lock().parent().unwrap();
            parent.kill(SignalFlags::SIGCHLD);
            println!("[kernel] One process has ended.");
        }
    } else {
        panic!("[kernel] Shutdown.");
    }
}

pub fn run_tasks() {
    loop {
        switch();
    }
}
