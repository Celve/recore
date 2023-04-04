use core::{cmp::min, mem::size_of};

use alloc::vec::Vec;
use fosix::fs::{DirEntry, FileStat, OpenFlags};

use crate::{
    fs::fileable::Fileable,
    io::{stdin::Stdin, stdout::Stdout},
    task::processor::fetch_curr_task,
};

use super::{open_dir, open_file, parse_str};

pub fn sys_read(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    if fd != 0 {
        panic!("[syscall] Doesn't support file read.");
    }
    let mut buffer = {
        let task = fetch_curr_task();
        let task_guard = task.lock();
        let page_table = task_guard.user_mem().page_table();
        page_table.translate_bytes(buffer_ptr.into(), buffer_len)
    };
    let stdin = Stdin;
    buffer.iter_mut().for_each(|b| **b = stdin.getchar() as u8);
    buffer_len as isize
}

pub fn sys_write(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    if fd != 1 {
        panic!("[syscall] Doesn't support file write.");
    }
    let buffer = fetch_curr_task()
        .lock()
        .user_mem()
        .page_table()
        .translate_bytes(buffer_ptr.into(), buffer_len);
    let stdout = Stdout;
    buffer.iter().for_each(|&&mut b| stdout.putchar(b));
    buffer_len as isize
}

pub fn sys_open(path: usize, flags: u32) -> isize {
    let flags = OpenFlags::from_bits(flags).unwrap();
    let cwd = fetch_curr_task().lock().cwd();
    let fd = if flags.contains(OpenFlags::DIR) {
        let path = &parse_str(path);
        let dir = open_dir(cwd, path);
        if dir.is_none() {
            return -1;
        }
        Fileable::Dir(dir.unwrap())
    } else {
        let file = open_file(cwd, &parse_str(path), flags);
        if file.is_none() {
            return -1;
        }
        Fileable::File(file.unwrap())
    };
    let task = fetch_curr_task();
    let mut task_guard = task.lock();
    task_guard.alloc_fd(fd) as isize
}

pub fn sys_close(fd: usize) -> isize {
    let task = fetch_curr_task();
    let mut task_guard = task.lock();
    let fd_table = task_guard.fd_table_mut();
    if fd > fd_table.len() {
        -1
    } else if fd_table[fd].is_none() {
        -1
    } else {
        fd_table[fd] = None;
        0
    }
}

pub fn sys_mkdir(dfd: usize, path: usize) -> isize {
    let path = parse_str(path);
    let dir = open_dir(
        *fetch_curr_task().lock().fd_table()[dfd]
            .as_ref()
            .unwrap()
            .as_dir()
            .unwrap(),
        &path,
    );
    if let Some(dir) = dir {
        dir.mkdir(&path).unwrap();
        0
    } else {
        -1
    }
}

pub fn sys_chdir(path: usize) -> isize {
    let path = parse_str(path);
    let dir = open_dir(fetch_curr_task().lock().cwd(), &path);
    if let Some(dir) = dir {
        let task = fetch_curr_task();
        let mut task_guard = task.lock();
        *task_guard.cwd_mut() = dir;
        0
    } else {
        -1
    }
}

pub fn sys_getdents(dfd: usize, des_ptr: usize, des_len: usize) -> isize {
    let task = fetch_curr_task();
    let task_guard = task.lock();
    let mut dst_bytes = task_guard
        .user_mem()
        .page_table()
        .translate_bytes(des_ptr.into(), des_len * size_of::<DirEntry>());

    let dir = open_dir(
        *task_guard.fd_table()[dfd]
            .as_ref()
            .unwrap()
            .as_dir()
            .unwrap(),
        ".",
    )
    .unwrap();

    let dir_entries = dir.to_dir_entries();
    let mut i = 0;
    for de in dir_entries {
        let src_bytes = de.as_bytes();
        for byte in src_bytes {
            *dst_bytes[i] = *byte;
            i += 1;
            if i >= dst_bytes.len() {
                return i as isize;
            }
        }
    }
    return i as isize;
}

pub fn sys_fstat(fd: usize, stat_ptr: usize) -> isize {
    let task = fetch_curr_task();
    let task_guard = task.lock();
    let mut dst_bytes = task_guard
        .user_mem()
        .page_table()
        .translate_bytes(stat_ptr.into(), size_of::<FileStat>());

    let dir = task_guard.fd_table()[fd].as_ref().unwrap();
    let stat = dir.stat();
    let src_bytes = stat.as_bytes();

    assert_eq!(src_bytes.len(), dst_bytes.len());
    for (i, byte) in dst_bytes.iter_mut().enumerate() {
        **byte = src_bytes[i];
    }

    0
}
