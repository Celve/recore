use alloc::vec::Vec;
use fosix::{
    fs::OpenFlags,
    signal::{SignalAction, SignalFlags, SIGCONT, SIGKILL, SIGSTOP},
};

use crate::task::{
    exit_yield, loader::get_app_data, manager::MANAGER, processor::fetch_curr_task, suspend_yield,
    task::TaskStatus,
};

use super::{open_file, parse_str};

pub fn sys_exit(exit_code: isize) -> isize {
    exit_yield(exit_code);
    0
}

pub fn sys_yield() -> isize {
    suspend_yield();
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

pub fn sys_exec(path: usize, mut args_ptr: *const usize) -> isize {
    let name = parse_str(path);
    let cwd = fetch_curr_task().lock().cwd();
    let file = open_file(cwd, &name, OpenFlags::RDONLY);
    if let Some(file) = file {
        println!("[kernel] Exec a new program.");

        // parse args
        let mut args = Vec::new();
        loop {
            let arg = {
                let page_table = fetch_curr_task().lock().page_table();
                page_table.translate_any::<usize>((args_ptr as usize).into())
            };
            if *arg == 0 {
                break;
            }
            let mut str = parse_str(*arg);
            str.push('\0');
            args.push(str);
            args_ptr = unsafe { args_ptr.add(1) };
        }

        fetch_curr_task().exec(file, &args);
        args.len() as isize // otherwise it would be overrided
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
        (pid == -1 || pid as usize == task.pid()) && task.task_status() == TaskStatus::Zombie
    });

    return if let Some(pos) = result {
        let removed_task = task_guard.children_mut().remove(pos);
        *task_guard
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

#[no_mangle]
pub fn sys_sigreturn() -> isize {
    let task = fetch_curr_task();
    let mut task_guard = task.lock();
    task_guard.sig_handling_mut().take();
    *task_guard.trap_ctx_mut() = task_guard.trap_ctx_backup_mut().take().unwrap();
    task_guard.trap_ctx().a0() as isize
}

pub fn sys_kill(pid: usize, sig: usize) -> isize {
    let manager = MANAGER.lock();
    let target = manager.iter().find(|task| {
        let task_guard = task.lock();
        task_guard.pid() == pid as usize && task_guard.pid() != 1
    });
    if let Some(task) = target {
        task.kill(SignalFlags::from_bits(1 << sig).unwrap());
        0
    } else {
        -1
    }
}

pub fn sys_sigaction(sig_id: usize, new_action_ptr: usize, old_action_ptr: usize) -> isize {
    if new_action_ptr == 0
        || old_action_ptr == 0
        || sig_id == SIGKILL as usize
        || sig_id == SIGSTOP as usize
        || sig_id == SIGCONT as usize
    {
        return -1;
    }

    let task = fetch_curr_task();
    let mut task_guard = task.lock();
    let page_table = task_guard.page_table();
    let new_action = page_table.translate_any::<SignalAction>(new_action_ptr.into());
    let old_action = page_table.translate_any::<SignalAction>(old_action_ptr.into());

    *old_action = task_guard.sig_actions()[sig_id];
    task_guard.sig_actions_mut()[sig_id] = *new_action;
    0
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    let task = fetch_curr_task();
    let mut task_guard = task.lock();
    if let Some(mask) = SignalFlags::from_bits(mask) {
        let old_mask = task_guard.sig_mask();
        *task_guard.sig_mask_mut() = mask;
        old_mask.bits() as isize
    } else {
        -1
    }
}
