mod com;
mod file;
mod process;

use alloc::{string::String, vec::Vec};
use fosix::fs::OpenFlags;

use crate::{
    fs::{dir::Dir, file::File, fuse::FUSE},
    task::processor::fetch_curr_task,
};

use self::{
    com::sys_pipe,
    file::{
        sys_chdir, sys_close, sys_fstat, sys_getdents, sys_lseek, sys_mkdir, sys_open, sys_read,
        sys_write,
    },
    process::{sys_exec, sys_exit, sys_fork, sys_waitpid, sys_yield},
};

const SYSCALL_MKDIR: usize = 34;
const SYSCALL_FSTAT: usize = 43;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_GETDENTS: usize = 61;
const SYSCALL_LSEEK: usize = 62;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
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
        SYSCALL_EXEC => sys_exec(args[0]),
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
        cwd = cwd.cd(step)?;
    }
    cwd.open(steps[steps.len() - 1], flags)
}

fn open_dir(cwd: Dir, path: &str) -> Option<Dir> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps.iter() {
        cwd = cwd.cd(step)?;
    }
    Some(cwd)
}

fn create_dir(cwd: Dir, path: &str) -> Option<Dir> {
    let (mut cwd, steps) = split_path(cwd, path);
    for step in steps[..steps.len() - 1].iter() {
        cwd = cwd.cd(step)?;
    }
    if cwd.mkdir(steps[steps.len() - 1]).is_err() {
        None
    } else {
        cwd.cd(steps[steps.len() - 1])
    }
}

fn parse_str(path: usize) -> String {
    fetch_curr_task()
        .lock()
        .user_mem()
        .page_table()
        .translate_str(path.into())
}
