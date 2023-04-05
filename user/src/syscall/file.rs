use fosix::fs::DirEntry;

use fosix::fs::FileStat;
use fosix::fs::OpenFlags;
use fosix::fs::SeekFlag;

use super::syscall;

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

pub fn sys_open(path: &str, flags: OpenFlags) -> isize {
    syscall(
        SYSCALL_OPEN,
        [path.as_ptr() as usize, flags.bits() as usize, 0],
    )
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_mkdir(dfd: usize, path: &str) -> isize {
    syscall(SYSCALL_MKDIR, [dfd, path.as_ptr() as usize, 0])
}

pub fn sys_chdir(path: &str) -> isize {
    syscall(SYSCALL_CHDIR, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_getdents(dfd: usize, des: &[DirEntry]) -> isize {
    syscall(SYSCALL_GETDENTS, [dfd, des.as_ptr() as usize, des.len()])
}

pub fn sys_fstat(fd: usize, stat: &mut FileStat) -> isize {
    syscall(SYSCALL_FSTAT, [fd, stat as *mut FileStat as usize, 0])
}

pub fn sys_lseek(fd: usize, offset: usize, flag: SeekFlag) -> isize {
    syscall(SYSCALL_LSEEK, [fd, offset, flag.bits() as usize])
}

pub fn sys_pipe(pipe: &mut [usize; 2]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
}
