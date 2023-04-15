use alloc::vec::Vec;
use fosix::{
    fs::OpenFlags,
    signal::{SignalAction, SignalFlags, SIGCONT, SIGKILL, SIGSTOP},
};

use crate::{
    proc::{manager::PROC_MANAGER, proc::ProcState},
    task::{
        exit_yield,
        manager::TASK_MANAGER,
        processor::{fetch_curr_proc, fetch_curr_task},
        suspend_yield,
        task::TaskState,
    },
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
    let proc = fetch_curr_proc().fork();
    let pid = proc.pid();
    let task = proc.lock().main_task();
    *task.lock().trap_ctx_mut().a0_mut() = 0;
    PROC_MANAGER.push(proc);
    TASK_MANAGER.push(task);
    println!("[kernel] Fork a new process with pid {}.", pid);
    pid as isize
}

pub fn sys_exec(path: usize, mut args_ptr: *const usize) -> isize {
    println!("[kernel] Try exec a new program.");
    let name = parse_str(path);
    let cwd = fetch_curr_proc().lock().cwd();
    let file = open_file(cwd, &name, OpenFlags::RDONLY);
    if let Some(file) = file {
        println!("[kernel] Exec a new program.");

        // parse args
        let mut args = Vec::new();
        loop {
            let arg = {
                let page_table = fetch_curr_proc().lock().page_table();
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

        fetch_curr_proc().exec(file, &args);
        args.len() as isize // otherwise it would be overrided
    } else {
        println!("[kernel] Fail to exec {}.", name);
        -1
    }
}

pub fn sys_getpid() -> isize {
    fetch_curr_proc().pid() as isize
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: usize) -> isize {
    let proc = fetch_curr_proc();
    let mut proc_guard = proc.lock();

    // find satisfied children
    let result = proc_guard.children().iter().position(|proc| {
        (pid == -1 || pid as usize == proc.pid()) && proc.lock().proc_status() == ProcState::Zombie
    });

    return if let Some(pos) = result {
        let removed_proc = proc_guard.children_mut().remove(pos);
        *proc_guard
            .page_table()
            .translate_any::<isize>(exit_code_ptr.into()) = removed_proc.lock().exit_code();
        let pid = removed_proc.pid() as isize;
        pid
    } else if proc_guard
        .children()
        .iter()
        .any(|task| pid == -1 || task.pid() == pid as usize)
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
    // let target = PROC_MANAGER.find(|task| {
    // let task_guard = task.lock();
    // task_guard.pid().id() == pid as usize && task_guard.pid().id() != 1
    // });
    let target = PROC_MANAGER.get(pid);
    if let Some(proc) = target {
        proc.kill(SignalFlags::from_bits(1 << sig).unwrap());
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

    let proc = fetch_curr_proc();
    let mut proc_guard = proc.lock();
    let page_table = proc_guard.page_table();
    let new_action = page_table.translate_any::<SignalAction>(new_action_ptr.into());
    let old_action = page_table.translate_any::<SignalAction>(old_action_ptr.into());

    *old_action = proc_guard.sig_actions()[sig_id];
    proc_guard.sig_actions_mut()[sig_id] = *new_action;
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
