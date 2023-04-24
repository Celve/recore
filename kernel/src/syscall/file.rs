use core::mem::size_of;

use fosix::fs::{DirEntry, FileStat, OpenFlags, SeekFlag};

use crate::{fs::fileable::Fileable, task::processor::Processor};

use super::{create_dir, open_dir, open_file, parse_str};

pub fn sys_read(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    let (mut fileable, mut seg) = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        let page_table = proc_guard.page_table();
        let fd_table = proc_guard.fd_table();
        let fileable = fd_table.get(fd).unwrap();
        (
            fileable,
            page_table.translate_segment(buffer_ptr.into(), buffer_len),
        )
    };

    fileable.read_seg(&mut seg) as isize
}

pub fn sys_write(fd: usize, buffer_ptr: usize, buffer_len: usize) -> isize {
    let (mut fileable, seg) = {
        let proc = Processor::curr_proc();
        let proc_guard = proc.lock();
        let page_table = proc_guard.page_table();
        let fd_table = proc_guard.fd_table();
        let fileable = fd_table.get(fd).unwrap();
        (
            fileable,
            page_table.translate_segment(buffer_ptr.into(), buffer_len),
        )
    };

    fileable.write_seg(&seg) as isize
}

pub fn sys_open(path: usize, flags: u32) -> isize {
    let flags = OpenFlags::from_bits(flags).unwrap();
    let cwd = Processor::curr_proc().lock().cwd();
    let fileable = if flags.contains(OpenFlags::DIR) {
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
    Processor::curr_proc().lock().fd_table_mut().alloc(fileable) as isize
}

pub fn sys_close(fd: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let fd_table = proc_guard.fd_table_mut();
    if fd > fd_table.len() {
        -1
    } else if fd_table.get(fd).is_none() {
        -1
    } else {
        fd_table.get_mut(fd).take();
        0
    }
}

pub fn sys_mkdir(dfd: usize, path: usize) -> isize {
    let path = parse_str(path);
    let dir = create_dir(
        Processor::curr_proc()
            .lock()
            .fd_table()
            .get(dfd)
            .unwrap()
            .as_dir()
            .unwrap(),
        &path,
    );
    if let Some(_) = dir {
        0
    } else {
        -1
    }
}

pub fn sys_chdir(path: usize) -> isize {
    let path = parse_str(path);
    let dir = open_dir(Processor::curr_proc().lock().cwd(), &path);
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
    let proc = Processor::curr_proc();
    let proc_guard = proc.lock();
    let mut dst_bytes = proc_guard
        .page_table()
        .translate_bytes(des_ptr.into(), des_len * size_of::<DirEntry>());

    let dir = open_dir(
        proc_guard.fd_table().get(dfd).unwrap().as_dir().unwrap(),
        ".",
    )
    .unwrap();

    let dir_entries = dir.lock().to_dir_entries();
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
    let proc = Processor::curr_proc();
    let proc_guard = proc.lock();
    let mut dst_bytes = proc_guard
        .page_table()
        .translate_bytes(stat_ptr.into(), size_of::<FileStat>());

    let dir = proc_guard.fd_table().get(fd).unwrap();
    let stat = dir.stat();
    let src_bytes = stat.as_bytes();

    assert_eq!(src_bytes.len(), dst_bytes.len());
    for (i, byte) in dst_bytes.iter_mut().enumerate() {
        **byte = src_bytes[i];
    }

    0
}

pub fn sys_lseek(fd: usize, offset: isize, flags: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let fd_table = proc_guard.fd_table_mut();
    let mut fileable = fd_table.get(fd).unwrap();
    fileable.seek(offset as usize, SeekFlag::from_bits(flags as u8).unwrap());
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let proc = Processor::curr_proc();
    let mut proc_guard = proc.lock();
    let fd_table = proc_guard.fd_table_mut();
    let fileable = fd_table.get(fd);
    if let Some(fileable) = fileable {
        fd_table.alloc(fileable) as isize
    } else {
        -1
    }
}
