use alloc::sync::{Arc, Weak};
use lazy_static::lazy_static;
use spin::Spin;

use crate::{
    fs::FUSE,
    proc::{manager::PROC_MANAGER, proc::Proc},
    task::{task::TaskState, timer::TIMER},
    time::get_time,
};

use super::{context::TaskContext, manager::TASK_MANAGER, task::Task};

pub struct Processor {
    /// The current task that the processor is exeucting.
    curr_task: Option<Weak<Task>>,

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

    pub fn curr_task(&self) -> Option<Arc<Task>> {
        self.curr_task.clone().and_then(|task| task.upgrade())
    }

    pub fn curr_task_take(&mut self) -> Option<Arc<Task>> {
        self.curr_task.take().and_then(|task| task.upgrade())
    }

    pub fn curr_task_mut(&mut self) -> &mut Option<Weak<Task>> {
        &mut self.curr_task
    }
}

lazy_static! {
    static ref PROCESSOR: Spin<Processor> = Spin::new(Processor::new());
}

/// Fetch the current task.
pub fn fetch_curr_task() -> Arc<Task> {
    PROCESSOR
        .lock()
        .curr_task()
        .expect("[kernel] There is no running task currently.")
}

/// Fetch the current process.
pub fn fetch_curr_proc() -> Arc<Proc> {
    fetch_curr_task().proc()
}

/// Fetch the idle task context pointer with processor locked.
pub fn fetch_idle_task_ctx_ptr() -> *mut TaskContext {
    PROCESSOR.lock().idle_task_ctx_ptr()
}

/// Switch from idle task to the next task.
///
/// When the next task yields, it will get into this function again.
pub fn switch() {
    let task = loop {
        let task = TASK_MANAGER.pop();
        if task.is_some() {
            break task;
        }
    };
    if let Some(task) = task {
        if task.lock().task_state() == TaskState::Ready {
            *task.lock().task_state_mut() = TaskState::Running;
        }

        let task_ctx = task.lock().task_ctx_ptr();
        let idle_task_ctx = PROCESSOR.lock().idle_task_ctx_ptr();
        *PROCESSOR.lock().curr_task_mut() = Some(task.phantom());

        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe {
            _switch(idle_task_ctx, task_ctx);
        }

        // check if timer is up
        TIMER.notify(get_time());
    } else {
        panic!("[kernel] Shutdown.");
    }
}

pub fn run_tasks() {
    let task = PROC_MANAGER.get(1).unwrap().lock().main_task();
    TASK_MANAGER.push(&task);
    FUSE.disk_manager().enable_non_blocking();
    loop {
        switch();
    }
}
