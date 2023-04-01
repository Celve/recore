mod file;
mod process;

use self::{
    file::{sys_read, sys_write},
    process::{sys_exec, sys_exit, sys_fork, sys_waitpid, sys_yield},
};

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
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
