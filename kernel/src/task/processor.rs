use core::arch::asm;

use alloc::sync::{Arc, Weak};
use lazy_static::lazy_static;
use spin::Spin;

use crate::{
    config::CPUS,
    proc::proc::Proc,
    task::{task::TaskState, timer::TIMER},
    time::get_time,
};

use super::{context::TaskContext, manager::TASK_MANAGER, task::Task};

#[derive(Default)]
pub struct Processor {
    /// The current task that the processor is exeucting.
    curr_task: Option<Weak<Task>>,

    /// A special task context that is used for thread switching.
    idle_task_ctx: TaskContext,
}

impl Processor {
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
    static ref PROCESSORS: [Spin<Processor>; CPUS] = Default::default();
}

pub fn hart_id() -> usize {
    let mut hart_id: usize;
    unsafe {
        asm!(
            "mv {hart_id}, tp",
            hart_id = out(reg) hart_id,
        );
    }
    hart_id
}

/// Fetch the current task.
pub fn fetch_curr_task() -> Arc<Task> {
    let task = PROCESSORS[hart_id()].lock().curr_task();
    if let Some(task) = task {
        task
    } else {
        panic!("[kernel] Hart {} has no running task currently.", hart_id())
    }
}

/// Fetch the current process.
pub fn fetch_curr_proc() -> Arc<Proc> {
    fetch_curr_task().proc()
}

/// Fetch the idle task context pointer with processor locked.
pub fn fetch_idle_task_ctx_ptr() -> *mut TaskContext {
    PROCESSORS[hart_id()].lock().idle_task_ctx_ptr()
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
        let idle_task_ctx = PROCESSORS[hart_id()].lock().idle_task_ctx_ptr();
        *PROCESSORS[hart_id()].lock().curr_task_mut() = Some(task.phantom());

        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe {
            _switch(idle_task_ctx, task_ctx);
        }

        if task.lock().task_state() == TaskState::Running {
            TASK_MANAGER.push(&task);
        }

        // check if timer is up
        TIMER.notify(get_time());
    }
}

#[no_mangle]
pub fn run_tasks() {
    loop {
        switch();
    }
}
