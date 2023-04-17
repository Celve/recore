use alloc::vec::Vec;
use fosix::{
    signal::{SignalAction, SignalFlags},
    syscall::*,
};

use crate::syscall::syscall;

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("[user] Return from syscall_exit()");
}

pub fn sys_yield() {
    syscall(SYSCALL_YIELD, [0; 3]);
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0; 3])
}

pub fn sys_exec(path: &str, args: &Vec<*const u8>) -> isize {
    syscall(
        SYSCALL_EXEC,
        [path.as_ptr() as usize, args.as_ptr() as usize, 0],
    )
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0; 3])
}

pub fn sys_waitpid(pid: isize, exit_code: &mut i32) -> isize {
    syscall(
        SYSCALL_WAITPID,
        [pid as usize, exit_code as *mut i32 as usize, 0],
    )
}

pub fn sys_kill(pid: usize, sig: usize) -> isize {
    syscall(SYSCALL_KILL, [pid, sig, 0])
}

pub fn sys_sigreturn() -> isize {
    syscall(SYSCALL_SIGRETURN, [0; 3])
}

pub fn sys_sigaction(
    sig_id: usize,
    new_action: &SignalAction,
    old_action: &mut SignalAction,
) -> isize {
    syscall(
        SYSCALL_SIGACTION,
        [
            sig_id,
            new_action as *const SignalAction as usize,
            old_action as *mut SignalAction as usize,
        ],
    )
}

pub fn sys_sigprocmask(mask: SignalFlags) -> isize {
    syscall(SYSCALL_SIGPROCMASK, [mask.bits() as usize, 0, 0])
}

pub fn sys_mutex_create(blocked: bool) -> isize {
    syscall(SYSCALL_MUTEX_CREATE, [blocked as usize, 0, 0])
}

pub fn sys_mutex_lock(id: usize) -> isize {
    syscall(SYSCALL_MUTEX_LOCK, [id, 0, 0])
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    syscall(SYSCALL_MUTEX_UNLOCK, [id, 0, 0])
}

pub fn sys_semaphore_create(counter: usize) -> isize {
    syscall(SYSCALL_SEMAPHORE_CREATE, [counter, 0, 0])
}

pub fn sys_semaphore_up(id: usize) -> isize {
    syscall(SYSCALL_SEMAPHORE_UP, [id, 0, 0])
}

pub fn sys_semaphore_down(id: usize) -> isize {
    syscall(SYSCALL_SEMAPHORE_DOWN, [id, 0, 0])
}

pub fn sys_condvar_create() -> isize {
    syscall(SYSCALL_CONDVAR_CREATE, [0; 3])
}

pub fn sys_condvar_wait(condvar_id: usize, lock_id: usize) -> isize {
    syscall(SYSCALL_CONDVAR_WAIT, [condvar_id, lock_id, 0])
}

pub fn sys_condvar_notify_one(id: usize) -> isize {
    syscall(SYSCALL_CONDVAR_NOTIFY_ONE, [id, 0, 0])
}

pub fn sys_condvar_notify_all(id: usize) -> isize {
    syscall(SYSCALL_CONDVAR_NOTIFY_ALL, [id, 0, 0])
}
