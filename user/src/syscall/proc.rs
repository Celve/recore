use alloc::vec::Vec;
use fosix::{signal::SignalAction, syscall::*};

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
