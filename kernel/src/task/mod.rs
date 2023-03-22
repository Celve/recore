use crate::{
    config::TRAMPOLINE_START_ADDRESS,
    mm::{address::VirPageNum, memory::KERNEL_SPACE},
    println,
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
    let curr_task_cell = fetch_curr_task();
    let mut curr_task = curr_task_cell.borrow_mut();
    *curr_task.task_status_mut() = TaskStatus::Zombie;
    *curr_task.exit_code_mut() = exit_code;
    schedule();
}

pub fn schedule() {
    let curr_task_cell = fetch_curr_task();
    let mut curr_task = curr_task_cell.borrow_mut();
    println!(
        "checking: {:#x}\n",
        usize::from(
            KERNEL_SPACE
                .borrow_mut()
                .page_table()
                .translate(VirPageNum::from(0x10000))
                .unwrap()
                .get_ppn()
        )
    );
    extern "C" {
        fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
    }
    unsafe { _switch(curr_task.task_ctx_mut(), fetch_idle_task_ctx()) }
}
