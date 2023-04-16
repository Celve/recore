use crate::{
    config::CLOCK_FREQ,
    task::{
        manager::TASK_MANAGER,
        processor::{fetch_curr_proc, fetch_curr_task},
        task::TaskState,
        timer::TIMER,
    },
    time::get_time,
};

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let proc = fetch_curr_proc();
    let task = proc.new_task(entry.into(), arg);

    TASK_MANAGER.push(&task);
    println!("Task manager has {}.", TASK_MANAGER.len());

    let tid = task.lock().tid();
    println!(
        "[kernel] Create thread {} with {:#x} and {:#x}.",
        tid, entry, arg
    );
    tid as isize
}

pub fn sys_gettid() -> isize {
    let tid = fetch_curr_proc().lock().main_task().lock().tid();
    tid as isize
}

pub fn sys_waittid(tid: isize, exit_code_ptr: usize) -> isize {
    let proc = fetch_curr_proc();
    let mut proc_guard = proc.lock();

    // find satisfied children
    let result = proc_guard.tasks().iter().position(|task| {
        (tid == -1 || tid as usize == task.lock().tid())
            && task.lock().task_state() == TaskState::Zombie
    });

    return if let Some(pos) = result {
        let removed_task = proc_guard.tasks_mut().remove(pos);
        *proc_guard
            .page_table()
            .translate_any::<isize>(exit_code_ptr.into()) = removed_task.lock().exit_code();
        let tid = removed_task.lock().tid() as isize;
        tid
    } else if proc_guard
        .tasks()
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
    let task = fetch_curr_task();
    task.stop();

    TIMER.subscribe(target_time, &task);
    0
}
