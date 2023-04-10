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

pub fn suspend_yield() {
    schedule();
}

pub fn stop_yield() {
    fetch_curr_task().stop();
    schedule();
}

pub fn exit_yield(exit_code: isize) {
    fetch_curr_task().exit(exit_code);
    schedule();
}

pub fn cont() {
    fetch_curr_task().cont();
}

pub fn schedule() {
    let task_ctx = fetch_curr_task().lock().task_ctx_ptr();
    extern "C" {
        fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
    }
    unsafe { _switch(task_ctx, fetch_idle_task_ctx_ptr()) }
}
