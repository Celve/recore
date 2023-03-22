use crate::{
    mm::{address::VirPageNum, memory::KERNEL_SPACE},
    task::{manager::fetch_idle_task_ctx, task::TaskContext},
};

use self::{manager::fetch_curr_task, task::TaskStatus};

mod loader;
pub mod manager;
pub mod pid;
pub mod stack;
mod switch;
pub mod task;

pub fn suspend_and_yield() {
    schedule();
}

pub fn exit_and_yield(exit_code: isize) {
    let curr_task = fetch_curr_task();
    *curr_task.lock().task_status_mut() = TaskStatus::Zombie;
    *curr_task.lock().exit_code_mut() = exit_code;
    drop(curr_task);
    schedule();
}

pub fn schedule() {
    let task_ctx = fetch_curr_task().lock().task_ctx_ptr();
    extern "C" {
        fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
    }
    unsafe { _switch(task_ctx, fetch_idle_task_ctx()) }
}
