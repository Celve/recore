use core::mem::size_of;

use alloc::vec::Vec;

use crate::{fs::fileable::Fileable, ipc::pipe::Pipe, task::processor::fetch_curr_task};

pub fn sys_pipe(pipe_ptr: usize) -> isize {
    let task = fetch_curr_task();
    let mut task_guard = task.lock();
    let fd_table = task_guard.fd_table_mut();

    let (pipe_read, pipe_write) = Pipe::new();
    let fd_read = fd_table.alloc(Fileable::Pipe(pipe_read));
    let fd_write = fd_table.alloc(Fileable::Pipe(pipe_write));
    let src_fds = [fd_read, fd_write];
    let src_bytes = unsafe {
        core::slice::from_raw_parts(&src_fds as *const _ as *const u8, size_of::<[usize; 2]>())
    };

    let mut dst_bytes = task_guard
        .user_mem()
        .page_table()
        .translate_bytes(pipe_ptr.into(), size_of::<[usize; 2]>());
    dst_bytes
        .iter_mut()
        .enumerate()
        .for_each(|(i, byte)| **byte = src_bytes[i]);

    0
}
