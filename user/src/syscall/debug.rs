use fosix::syscall::SYSCALL_PROCDUMP;

use super::syscall;

pub fn sys_procdump() -> isize {
    syscall(SYSCALL_PROCDUMP, [0, 0, 0])
}
