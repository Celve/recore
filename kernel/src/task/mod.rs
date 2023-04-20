use crate::task::{
    context::TaskContext,
    processor::{fetch_curr_task, fetch_idle_task_ctx_ptr},
};

use self::manager::TASK_MANAGER;

pub mod context;
pub mod manager;
pub mod processor;
pub mod task;
pub mod timer;

pub fn schedule() {
    let task_ctx = fetch_curr_task().lock().task_ctx_ptr();
    extern "C" {
        fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
    }
    unsafe { _switch(task_ctx, fetch_idle_task_ctx_ptr()) }
}
