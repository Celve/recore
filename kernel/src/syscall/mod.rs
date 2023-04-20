mod com;
mod file;
mod proc;
mod task;

use alloc::{string::String, vec::Vec};
use fosix::{fs::OpenFlags, syscall::*};
use fs::{dir::Dir, file::File};

use crate::{drivers::blockdev::BlkDev, fs::FUSE, task::processor::fetch_curr_proc};

use self::{com::sys_pipe, file::*, proc::*, task::*};

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_MKDIR => sys_mkdir(args[0], args[1]),
        SYSCALL_FSTAT => sys_fstat(args[0], args[1]),
        SYSCALL_CHDIR => sys_chdir(args[0]),
        SYSCALL_OPEN => sys_open(args[0], args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0]),
        SYSCALL_GETDENTS => sys_getdents(args[0], args[1], args[2]),
        SYSCALL_LSEEK => sys_lseek(args[0], args[1] as isize, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1], args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1], args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as isize),
        SYSCALL_SLEEP => sys_sleep(args[0] as usize),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_KILL => sys_kill(args[0], args[1]),
        SYSCALL_SIGACTION => sys_sigaction(args[0], args[1], args[2]),
        SYSCALL_SIGPROCMASK => sys_sigprocmask(args[0] as u32),
        SYSCALL_SIGRETURN => sys_sigreturn(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0], args[1] as *const usize),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1]),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0] as isize, args[1]),
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEMAPHORE_CREATE => sys_semaphore_create(args[0]),
        SYSCALL_SEMAPHORE_UP => sys_semaphore_up(args[0]),
        SYSCALL_SEMAPHORE_DOWN => sys_semaphore_down(args[0]),
        SYSCALL_CONDVAR_CREATE => sys_condvar_create(),
        SYSCALL_CONDVAR_WAIT => sys_condvar_wait(args[0], args[1]),
        SYSCALL_CONDVAR_NOTIFY_ONE => sys_condvar_notify_one(args[0]),
        SYSCALL_CONDVAR_NOTIFY_ALL => sys_condvar_notify_all(args[0]),
        _ => panic!("[kernel] Unknown syscall id: {}", id),
    }
}

fn normalize_path(path: &str) -> &str {
    if path.ends_with("/") {
        &path[..path.len() - 1]
    } else {
        path
    }
}

fn split_path(cwd: Dir<BlkDev>, path: &str) -> (Dir<BlkDev>, Vec<&str>) {
    let path = normalize_path(path);
    if path.starts_with("/") {
        // absolute path
        let steps: Vec<&str> = path[1..].split("/").collect();
        let cwd = FUSE.root();
        (cwd, steps)
    } else {
        // relative path
        let steps: Vec<&str> = path.split("/").collect();
        (cwd, steps)
    }
}

fn open_file(cwd: Dir<BlkDev>, path: &str, flags: OpenFlags) -> Option<File<BlkDev>> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps[..steps.len() - 1].iter() {
        let temp = cwd.lock().cd(step)?;
        cwd = temp;
    }
    let temp = cwd.lock().open(steps[steps.len() - 1], flags);
    temp
}

fn open_dir(cwd: Dir<BlkDev>, path: &str) -> Option<Dir<BlkDev>> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps.iter() {
        let temp = cwd.lock().cd(step)?;
        cwd = temp;
    }
    Some(cwd)
}

fn create_dir(cwd: Dir<BlkDev>, path: &str) -> Option<Dir<BlkDev>> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps[..steps.len() - 1].iter() {
        let temp = cwd.lock().cd(step)?;
        cwd = temp;
    }
    if cwd.lock().mkdir(steps[steps.len() - 1]).is_err() {
        None
    } else {
        cwd.lock().cd(steps[steps.len() - 1])
    }
}

fn parse_str(path: usize) -> String {
    fetch_curr_proc()
        .lock()
        .page_table()
        .translate_str(path.into())
}
