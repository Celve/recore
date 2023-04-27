use core::arch::asm;

use alloc::sync::{Arc, Weak};
use lazy_static::lazy_static;
use spin::Spin;

use crate::{
    config::{CPUS, PELT_PERIOD, SCHED_PERIOD},
    proc::proc::Proc,
    sync::mcs::Mcs,
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

    /// PELT tag
    pelt_period: usize,
}

lazy_static! {
    pub static ref PROCESSORS: [Mcs<Processor>; CPUS] = Default::default();
}

impl Processor {
    fn glob_curr_task() -> Option<Arc<Task>> {
        PROCESSORS[Processor::hart_id()].lock().local_curr_task()
    }

    pub fn curr_task() -> Arc<Task> {
        Processor::glob_curr_task().unwrap()
    }

    fn glob_curr_proc() -> Option<Arc<Proc>> {
        PROCESSORS[Processor::hart_id()]
            .lock()
            .local_curr_task()
            .map(|task| task.proc())
    }

    pub fn curr_proc() -> Arc<Proc> {
        Processor::glob_curr_proc().unwrap()
    }

    pub fn idle_task_ctx_ptr() -> *mut TaskContext {
        PROCESSORS[Processor::hart_id()]
            .lock()
            .local_idle_task_ctx_ptr()
    }

    pub fn curr_processor() -> &'static Mcs<Processor> {
        &PROCESSORS[Processor::hart_id()]
    }

    /// Yield the task.
    ///
    /// It's not like the `suspend()', because it would be put into the task manager when called.
    pub fn yield_now() {
        {
            let task = Processor::curr_task();
            task.lock().task_time_mut().runout();
        }
        Processor::switch();
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
        Processor::switch();
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
        Processor::switch();
    }

    /// Schedule the task, namely giving back control to the processor's idle thread.
    fn switch() {
        let task_ctx = Processor::curr_task().lock().task_ctx_ptr();
        extern "C" {
            fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
        }
        unsafe { _switch(task_ctx, Processor::idle_task_ctx_ptr()) }
    }

    /// The entry point of processor.
    pub fn run_tasks() {
        loop {
            Processor::schedule();
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
    pub fn schedule() {
        let task = Processor::curr_processor().lock().pop();
        if let Some((task, time, _)) = task {
            if task.lock().task_state() == TaskState::Ready {
                *task.lock().task_state_mut() = TaskState::Running;
            }

            // set up rest time
            task.lock().task_time_mut().setup(time);

            let task_ctx = task.lock().task_ctx_ptr();
            let idle_task_ctx = {
                let mut processor = PROCESSORS[Processor::hart_id()].lock();
                *processor.local_curr_task_mut() = Some(task.phantom());
                processor.local_idle_task_ctx_ptr()
            };

            extern "C" {
                fn _switch(curr_ctx: *mut TaskContext, next_ctx: *const TaskContext);
            }
            unsafe {
                _switch(idle_task_ctx, task_ctx);
            }

            let mut processor = PROCESSORS[Processor::hart_id()].lock();
            if task.lock().task_state() == TaskState::Running {
                processor.push_normal(&task);
            }

            // check if timer is up
            TIMER.notify(get_time());
        } else {
            // // try to steal task from other processor
            // let mut curr = Processor::curr_processor().lock();
            // for id in 0..CPUS {
            //     if id != Processor::hart_id() {
            //         let other = PROCESSORS[id].try_lock();
            //         if let Some(mut other) = other {
            //             if let Some((task, _, is_realtime)) = other.pop() {
            //                 curr.push(&task, is_realtime);
            //                 println!(
            //                     "[kernel] Balance: move task {} from {} to {}",
            //                     task.proc().pid(),
            //                     id,
            //                     Processor::hart_id()
            //                 );
            //                 break;
            //             }
            //         }
            //     }
            // }

            // if curr.scheduler.len() == 0 {
            //     drop(curr);
            //     sleep(SCHED_PERIOD);
            // }
        }

        let mut processor = Processor::curr_processor().lock();
        let now = pelt_period(get_time());
        if processor.pelt_period != now {
            processor.pelt_period = now;
            if now % CPUS == Processor::hart_id() {
                drop(processor);
                Processor::balance();
            }
        }
    }

    /// The function is used to balance the load of all processors. It adapts an `O(n)` algorithm.
    ///
    /// Its basic idea is steal. A processor try to steal tasks from another processor to make the two balanced.
    pub fn balance() {
        let curr_hart = Processor::hart_id();
        let next_hart = (curr_hart + (get_time() % (CPUS - 1)) + 1) % CPUS;

        let (mut recv, mut send) = if curr_hart < next_hart {
            let curr_processor = PROCESSORS[curr_hart].lock();
            let next_processor = PROCESSORS[next_hart].lock();
            (curr_processor, next_processor)
        } else {
            let next_processor = PROCESSORS[next_hart].lock();
            let curr_processor = PROCESSORS[curr_hart].lock();
            (curr_processor, next_processor)
        };

        // really naive implementation
        while let Some((task, _, is_realtime)) = send.pop() {
            if task.lock().task_time().history_load() + recv.load() <= send.load() {
                task.lock().task_time_mut().clear(); // to make it the first
                recv.push(&task, is_realtime);
            } else {
                send.push(&task, is_realtime);
                break;
            }
        }
    }

    pub fn procdump() {
        println!("[kernel] Processor dump: ");
        for id in 0..CPUS {
            let processor = PROCESSORS[id].lock();
            println!("\tHart {}: ", id);
            if let Some(task) = processor.local_curr_task() {
                let task_guard = task.lock();
                println!(
                    "\t\tRunning: pid {} vruntime {} load {}",
                    task.proc().pid(),
                    task_guard.task_time().vruntime(),
                    task_guard.task_time().history_load()
                );
            }
            processor.scheduler.iter().for_each(|sched_entity| {
                let task = sched_entity.task().upgrade();
                if let Some(task) = task {
                    let task_guard = task.lock();
                    println!(
                        "\t\tWaiting: pid {} vruntime {} load {}",
                        task.proc().pid(),
                        task_guard.task_time().vruntime(),
                        task_guard.task_time().history_load()
                    );
                }
            })
        }
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

impl Processor {
    pub fn push_realtime(&mut self, task: &Arc<Task>) {
        self.scheduler.push(task, true);
    }

    pub fn push_normal(&mut self, task: &Arc<Task>) {
        self.scheduler.push(task, false);
    }

    pub fn push(&mut self, task: &Arc<Task>, is_realtime: bool) {
        if is_realtime {
            self.push_realtime(task);
        } else {
            self.push_normal(task);
        }
    }

    /// Return the task and its vruntime.
    pub fn pop(&mut self) -> Option<(Arc<Task>, usize, bool)> {
        self.scheduler.pop()
    }

    pub fn load(&self) -> usize {
        self.scheduler.load()
    }
}

fn pelt_period(now: usize) -> usize {
    now / PELT_PERIOD
}
