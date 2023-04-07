use fosix::{fs::*, syscall::*};

use super::syscall;

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

pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0])
}
