use fosix::syscall::{SYSCALL_SHUTDOWN, SYSCALL_TIME};

use super::syscall;

pub fn sys_shutdown(exit_code: usize) -> ! {
    syscall(SYSCALL_SHUTDOWN, [exit_code, 0, 0]);
    panic!("[user] User executed when system is down.");
}

pub fn sys_time() -> isize {
    syscall(SYSCALL_TIME, [0, 0, 0])
}
