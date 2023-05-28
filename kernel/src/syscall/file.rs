use core::mem::size_of;

use fosix::fs::{DirEntry, FileStat, OpenFlags, SeekFlag};

use crate::{fs::fileable::Fileable, task::processor::Processor};

use super::{create_dir, open_dir, open_file, parse_str};

pub fn sys_read(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    let (mut fileable, mut seg) = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        let page_table = proc_guard.page_table();
        let fileable = proc_guard.fd_table.get(fd).unwrap();
        let seg = unsafe { page_table.translate_segment(buffer_ptr.into(), buffer_len) };
        (fileable, seg)
    };

    fileable.read_seg(&mut seg) as isize
}

pub fn sys_write(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    let (mut fileable, seg) = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        let page_table = proc_guard.page_table();
        let fileable = proc_guard.fd_table.get(fd).unwrap();
        let seg = unsafe { page_table.translate_segment(buffer_ptr.into(), buffer_len) };
        (fileable, seg)
    };

    fileable.write_seg(&seg) as isize
}

pub fn sys_open(path: usize, flags: u32) -> isize {
    let flags = OpenFlags::from_bits(flags).unwrap();
    let cwd = Processor::curr_proc().lock().cwd();
    let path = &unsafe { parse_str(path.into()) };
    let fileable = if flags.contains(OpenFlags::DIR) {
        let dir = open_dir(cwd, path);
        if dir.is_none() {
            return -1;
        }
        Fileable::Dir(dir.unwrap())
    } else {
        let file = open_file(cwd, path, flags);
        if file.is_none() {
            return -1;
        }
        Fileable::File(file.unwrap())
    };
    Processor::curr_proc().lock().fd_table.alloc(fileable) as isize
}

pub fn sys_close(fd: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let flag = proc_guard.fd_table.dealloc(fd);
    if flag {
        0
    } else {
        -1
    }
}

pub fn sys_mkdir(dfd: usize, path: usize) -> isize {
    let path = unsafe { parse_str(path.into()) };
    let dir = create_dir(
        Processor::curr_proc()
            .lock()
            .fd_table
            .get(dfd)
            .unwrap()
            .as_dir()
            .unwrap(),
        &path,
    );
    if dir.is_some() {
        0
    } else {
        -1
    }
}

pub fn sys_chdir(path: usize) -> isize {
    let path = unsafe { parse_str(path.into()) };
    let cwd = Processor::curr_proc().lock().cwd();
    let dir = open_dir(cwd, &path);
    if let Some(dir) = dir {
        let proc = Processor::curr_proc();
        let mut proc_guard = proc.lock();
        *proc_guard.cwd_mut() = dir;
        0
    } else {
        -1
    }
}

pub fn sys_getdents(dfd: usize, des_ptr: usize, des_len: usize) -> isize {
    let cwd = Processor::curr_proc()
        .lock()
        .fd_table
        .get(dfd)
        .unwrap()
        .as_dir()
        .unwrap();
    let dir = open_dir(cwd, ".").unwrap();
    let dir_entries = dir.lock().to_dir_entries();

    let proc = Processor::curr_proc();
    let proc_guard = proc.lock();
    let mut dst_bytes = unsafe {
        proc_guard
            .page_table()
            .translate_bytes(des_ptr.into(), des_len * size_of::<DirEntry>())
    };

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
    i as isize
}

pub fn sys_fstat(fd: usize, stat_ptr: usize) -> isize {
    let dir = Processor::curr_proc().lock().fd_table.get(fd).unwrap();
    let stat = dir.stat();
    let src_bytes = stat.as_bytes();

    let proc = Processor::curr_proc();
    let proc_guard = proc.lock();
    let mut dst_bytes = unsafe {
        proc_guard
            .page_table()
            .translate_bytes(stat_ptr.into(), size_of::<FileStat>())
    };

    assert_eq!(src_bytes.len(), dst_bytes.len());
    for (i, byte) in dst_bytes.iter_mut().enumerate() {
        **byte = src_bytes[i];
    }

    0
}

pub fn sys_lseek(fd: usize, offset: isize, flags: usize) -> isize {
    let mut fileable = Processor::curr_proc().lock().fd_table.get(fd).unwrap();
    fileable.seek(offset as usize, SeekFlag::from_bits(flags as u8).unwrap());
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let fd_table = &mut proc_guard.fd_table;
    let fileable = fd_table.get(fd);
    if let Some(fileable) = fileable {
        fd_table.alloc(fileable) as isize
    } else {
        -1
    }
}
