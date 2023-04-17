use bitflags::bitflags;

pub const SYSCALL_DUP: usize = 24;
pub const SYSCALL_MKDIR: usize = 34;
pub const SYSCALL_FSTAT: usize = 43;
pub const SYSCALL_CHDIR: usize = 49;
pub const SYSCALL_OPEN: usize = 56;
pub const SYSCALL_CLOSE: usize = 57;
pub const SYSCALL_PIPE: usize = 59;
pub const SYSCALL_GETDENTS: usize = 61;
pub const SYSCALL_LSEEK: usize = 62;
pub const SYSCALL_READ: usize = 63;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_SLEEP: usize = 101;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_KILL: usize = 129;
pub const SYSCALL_SIGACTION: usize = 134;
pub const SYSCALL_SIGPROCMASK: usize = 135;
pub const SYSCALL_SIGRETURN: usize = 139;
pub const SYSCALL_FORK: usize = 220;
pub const SYSCALL_EXEC: usize = 221;
pub const SYSCALL_WAITPID: usize = 260;
pub const SYSCALL_THREAD_CREATE: usize = 1000;
pub const SYSCALL_GETTID: usize = 1001;
pub const SYSCALL_WAITTID: usize = 1002;
pub const SYSCALL_MUTEX_CREATE: usize = 1010;
pub const SYSCALL_MUTEX_LOCK: usize = 1011;
pub const SYSCALL_MUTEX_UNLOCK: usize = 1012;
pub const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
pub const SYSCALL_SEMAPHORE_UP: usize = 1021;
pub const SYSCALL_SEMAPHORE_DOWN: usize = 1022;

bitflags! {
    pub struct WaitFlags: u8 {
        const NOHANG = 1 << 0;
    }
}
