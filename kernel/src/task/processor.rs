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
    fn glob_curr_task() -> Option<Arc<Task>> {
        PROCESSORS[hart_id()].lock().local_curr_task()
    }

    pub fn curr_task() -> Arc<Task> {
        Processor::glob_curr_task().unwrap()
    }

    fn glob_curr_proc() -> Option<Arc<Proc>> {
        PROCESSORS[hart_id()]
            .lock()
            .local_curr_task()
            .map(|task| task.proc())
    }

    pub fn curr_proc() -> Arc<Proc> {
        Processor::glob_curr_proc().unwrap()
    }

    pub fn idle_task_ctx_ptr() -> *mut TaskContext {
        PROCESSORS[hart_id()].lock().local_idle_task_ctx_ptr()
    }

    /// Yield the task.
    ///
    /// It's not like the `suspend()', because it would be put into the task manager when called.
    pub fn yield_now() {
        {
            let task = Processor::curr_task();
            task.lock().task_time_mut().runout();
        }
        Processor::schedule();
    }

    /// Suspend the task.
    ///
    /// When `suspend()` is called, the task would never be put into the task manager again.
    /// There should be other structure that holds the task, and it should wake up the task when needed.
    pub fn suspend() {
        {
            let task = Processor::curr_task();
            *task.lock().task_state_mut() = TaskState::Stopped;
        }
        Processor::schedule();
    }

    /// Exit the task.
    ///
    /// It directly exits the task by setting the state and exit code.
    /// It's illegal to put this task to the task manager again.
    pub fn exit(exit_code: isize) {
        {
            let task = Processor::curr_task();
            println!(
                "process {} thread {} exit with code {}",
                task.proc().pid(),
                task.lock().tid(),
                exit_code
            );
            task.exit(exit_code);
        }
        Processor::schedule();
    }

    /// Schedule the task, namely giving back control to the processor's idle thread.
    fn schedule() {
        let task_ctx = Processor::curr_task().lock().task_ctx_ptr();
        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe { _switch(task_ctx, Processor::idle_task_ctx_ptr()) }
    }
}

impl Processor {
    pub fn local_idle_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_ctx
    }

    pub fn local_curr_task(&self) -> Option<Arc<Task>> {
        self.curr_task.clone().and_then(|task| task.upgrade())
    }

    pub fn local_curr_task_take(&mut self) -> Option<Arc<Task>> {
        self.curr_task.take().and_then(|task| task.upgrade())
    }

    pub fn local_curr_task_mut(&mut self) -> &mut Option<Weak<Task>> {
        &mut self.curr_task
    }

    pub fn local_scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn local_scheduler_mut(&mut self) -> &mut Scheduler {
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
            *processor.local_curr_task_mut() = Some(task.phantom());
            processor.local_idle_task_ctx_ptr()
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
