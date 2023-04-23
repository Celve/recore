use core::arch::asm;

use alloc::sync::{Arc, Weak};
use lazy_static::lazy_static;
use spin::Spin;

use crate::{
    config::{CPUS, SCHED_PERIOD},
    proc::proc::Proc,
    task::{task::TaskState, timer::TIMER},
    time::{get_time, sleep},
};

use super::{context::TaskContext, scheduler::Scheduler, task::Task};

#[derive(Default)]
pub struct Processor {
    /// The current task that the processor is exeucting.
    curr_task: Option<Weak<Task>>,

    /// Scheduling the tasks for the processor.
    scheduler: Scheduler,

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

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }
}

lazy_static! {
    pub static ref PROCESSORS: [Spin<Processor>; CPUS] = Default::default();
}

impl Processor {
    pub fn push(&mut self, task: &Arc<Task>) {
        self.scheduler.push(task);
    }
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
    let task = PROCESSORS[hart_id()].lock().scheduler.pop();
    if let Some((task, time)) = task {
        if task.lock().task_state() == TaskState::Ready {
            *task.lock().task_state_mut() = TaskState::Running;
        }

        // set up rest time
        task.lock().task_time_mut().setup(time);

        let task_ctx = task.lock().task_ctx_ptr();
        let idle_task_ctx = {
            let mut processor = PROCESSORS[hart_id()].lock();
            *processor.curr_task_mut() = Some(task.phantom());
            processor.idle_task_ctx_ptr()
        };

        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe {
            _switch(idle_task_ctx, task_ctx);
        }

        let mut processor = PROCESSORS[hart_id()].lock();
        if task.lock().task_state() == TaskState::Running {
            processor.scheduler.push(&task);
        }

        // check if timer is up
        TIMER.notify(get_time());
    } else {
        // try to steal task from other processor
        let mut curr = PROCESSORS[hart_id()].lock();
        for id in 0..CPUS {
            if id != hart_id() {
                let mut other = PROCESSORS[id].lock();
                if let Some((task, _)) = other.scheduler.pop() {
                    curr.scheduler.push(&task);
                    break;
                }
            }
        }

        if curr.scheduler.len() == 0 {
            sleep(SCHED_PERIOD);
        }
    }
}

#[no_mangle]
pub fn run_tasks() {
    loop {
        switch();
    }
}
