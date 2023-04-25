use fosix::syscall::{SYSCALL_GETTID, SYSCALL_SLEEP, SYSCALL_THREAD_CREATE, SYSCALL_WAITTID};

use super::syscall;

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    syscall(SYSCALL_THREAD_CREATE, [entry, arg, 0])
}

pub fn sys_gettid() -> isize {
    syscall(SYSCALL_GETTID, [0, 0, 0])
}

pub fn sys_waittid(tid: isize, exit_code_ptr: usize) -> isize {
    syscall(SYSCALL_WAITTID, [tid as usize, exit_code_ptr, 0])
}

pub fn sys_sleep(ms: usize) -> isize {
    syscall(SYSCALL_SLEEP, [ms, 0, 0])
}
