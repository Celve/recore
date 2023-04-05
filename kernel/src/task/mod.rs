use crate::task::{processor::fetch_idle_task_ctx_ptr, task::TaskContext};

use self::{processor::fetch_curr_task, task::TaskStatus};

pub mod fd_table;
pub mod loader;
pub mod manager;
pub mod pid;
pub mod processor;
pub mod stack;
mod switch;
pub mod task;

pub fn suspend_and_yield() {
    schedule();
}

pub fn exit_and_yield(exit_code: isize) {
    // modify task status and exit code
    {
        let curr_task = fetch_curr_task();
        let mut curr_task_guard = curr_task.lock();
        *curr_task_guard.task_status_mut() = TaskStatus::Zombie;
        *curr_task_guard.exit_code_mut() = exit_code;
    }

    schedule();
}

pub fn schedule() {
    let task_ctx = fetch_curr_task().lock().task_ctx_ptr();
    extern "C" {
        fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
    }
    unsafe { _switch(task_ctx, fetch_idle_task_ctx_ptr()) }
}
