use crate::{
    config::CLOCK_FREQ,
    task::{
        processor::{Processor, PROCESSORS},
        task::TaskStatus,
        timer::TIMER,
    },
    time::get_time,
};

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let proc = Processor::curr_proc();
    let task = proc.new_task(entry.into(), arg);

    PROCESSORS[Processor::hart_id()].lock().push_normal(&task);

    let tid = task.lock().tid();
    infoln!("Process {} has created new thread {}", proc.pid(), tid);
    tid as isize
}

pub fn sys_gettid() -> isize {
    let tid = Processor::curr_proc().lock().main_task().lock().tid();
    tid as isize
}

pub fn sys_waittid(tid: isize, exit_code_ptr: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();

    // find satisfied children
    let result = proc_guard.tasks.iter().position(|task| {
        (tid == -1 || tid as usize == task.lock().tid())
            && task.lock().task_status == TaskStatus::Zombie
    });

    return if let Some(pos) = result {
        let removed_task = proc_guard.tasks.remove(pos);
        unsafe {
            *proc_guard
                .page_table()
                .translate_any::<isize>(exit_code_ptr.into()) = removed_task.lock().exit_code;
        }
        let tid = removed_task.lock().tid() as isize;
        tid
    } else if proc_guard
        .tasks
        .iter()
        .any(|task| tid == -1 || task.lock().tid() == tid as usize)
    {
        -2
    } else {
        -1
    };
}

/// Sleep the thread for the specified millisecond.
///
/// In the current design, the sleep would be interrupted by a external signal SIGCONT.
pub fn sys_sleep(time_ms: usize) -> isize {
    let curr_time = get_time();
    let target_time = curr_time + CLOCK_FREQ * time_ms / 1000;
    let task = Processor::curr_task();
    TIMER.subscribe(target_time, &task);
    Processor::suspend();
    0
}
