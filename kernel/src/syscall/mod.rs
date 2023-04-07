mod com;
mod file;
mod process;

use alloc::{string::String, vec::Vec};
use fosix::{fs::OpenFlags, syscall::*};

use crate::{
    fs::{dir::Dir, file::File, fuse::FUSE},
    task::processor::fetch_curr_task,
};

use self::{com::sys_pipe, file::*, process::*};

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
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0], args[1] as *const usize),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1]),
        _ => todo!(),
    }
}

fn normalize_path(path: &str) -> &str {
    if path.ends_with("/") {
        &path[..path.len() - 1]
    } else {
        path
    }
}

fn split_path(cwd: Dir, path: &str) -> (Dir, Vec<&str>) {
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

fn open_file(cwd: Dir, path: &str, flags: OpenFlags) -> Option<File> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps[..steps.len() - 1].iter() {
        let temp = cwd.lock().cd(step)?;
        cwd = temp;
    }
    let temp = cwd.lock().open(steps[steps.len() - 1], flags);
    temp
}

fn open_dir(cwd: Dir, path: &str) -> Option<Dir> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps.iter() {
        let temp = cwd.lock().cd(step)?;
        cwd = temp;
    }
    Some(cwd)
}

fn create_dir(cwd: Dir, path: &str) -> Option<Dir> {
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
    fetch_curr_task()
        .lock()
        .user_mem()
        .page_table()
        .translate_str(path.into())
}
