use crate::task::{
    context::TaskContext,
    processor::{fetch_curr_task, fetch_idle_task_ctx_ptr},
};

use self::processor::fetch_curr_proc;

pub mod context;
pub mod manager;
pub mod processor;
pub mod task;

pub fn suspend_yield() {
    schedule();
}

pub fn stop_yield() {
    fetch_curr_proc().stop();
    schedule();
}

pub fn exit_yield(exit_code: isize) {
    fetch_curr_proc().exit(exit_code);
    schedule();
}

pub fn cont() {
    fetch_curr_proc().cont();
}

pub fn schedule() {
    let task_ctx = fetch_curr_task().lock().task_ctx_ptr();
    extern "C" {
        fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
    }
    unsafe { _switch(task_ctx, fetch_idle_task_ctx_ptr()) }
}
