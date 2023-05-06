use core::mem::size_of;

use crate::{fs::fileable::Fileable, ipc::pipe::Pipe, task::processor::Processor};

pub fn sys_pipe(pipe_ptr: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let fd_table = proc_guard.fd_table_mut();

    let (pipe_read, pipe_write) = Pipe::new();
    let fd_read = fd_table.alloc(Fileable::Pipe(pipe_read));
    let fd_write = fd_table.alloc(Fileable::Pipe(pipe_write));
    let src_fds = [fd_read, fd_write];
    let src_bytes = unsafe {
        core::slice::from_raw_parts(&src_fds as *const _ as *const u8, size_of::<[usize; 2]>())
    };

    let mut dst_bytes = unsafe {
        proc_guard
            .page_table()
            .translate_bytes(pipe_ptr.into(), size_of::<[usize; 2]>())
    };
    dst_bytes
        .iter_mut()
        .enumerate()
        .for_each(|(i, byte)| **byte = src_bytes[i]);

    0
}
