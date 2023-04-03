use fosix::fs::OpenFlags;

use crate::task::{
    exit_and_yield, loader::get_app_data, manager::MANAGER, processor::fetch_curr_task,
    suspend_and_yield, task::TaskStatus,
};

use super::{open_file, parse_str};

pub fn sys_exit(exit_code: isize) -> isize {
    exit_and_yield(exit_code);
    0
}

pub fn sys_yield() -> isize {
    suspend_and_yield();
    0
}

pub fn sys_fork() -> isize {
    let task = fetch_curr_task().fork();
    let pid = task.lock().pid();
    *task.lock().trap_ctx_mut().a0_mut() = 0;
    MANAGER.lock().push(task);
    println!("[kernel] Fork a new process with pid {}.", pid);
    pid as isize
}

pub fn sys_exec(path: usize) -> isize {
    let name = parse_str(path);
    let cwd = fetch_curr_task().lock().cwd();
    let file = open_file(cwd, &name, OpenFlags::RDONLY);
    if let Some(file) = file {
        println!("[kernel] Exec a new program.");
        fetch_curr_task().exec(file);
        0
    } else {
        println!("[kernel] Fail to exec {}.", name);
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: usize) -> isize {
    let task = fetch_curr_task();
    let mut task_guard = task.lock();

    // find satisfied children
    let result = task_guard.children().iter().position(|task| {
        let task = task.lock();
        (pid == -1 || pid as usize == task.pid()) && *task.task_status() == TaskStatus::Zombie
    });

    return if let Some(pos) = result {
        let removed_task = task_guard.children_mut().remove(pos);
        *task_guard
            .user_mem()
            .page_table()
            .translate_any::<isize>(exit_code_ptr.into()) = removed_task.lock().exit_code();
        let pid = removed_task.lock().pid() as isize;
        pid
    } else if task_guard
        .children()
        .iter()
        .any(|task| pid == -1 || task.lock().pid() == pid as usize)
    {
        -2
    } else {
        -1
    };
}
