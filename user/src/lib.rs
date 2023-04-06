#![no_std]
#![no_main]
#![feature(linkage, panic_info_message)]

extern crate alloc;

pub mod complement;
pub mod console;
pub mod syscall;

use alloc::vec::Vec;
use allocator::heap::LockedBuddyHeap;
use bitflags::bitflags;
use fosix::fs::{DirEntry, FileStat, OpenFlags, SeekFlag};
use syscall::{
    file::{
        sys_chdir, sys_close, sys_fstat, sys_getdents, sys_lseek, sys_mkdir, sys_open, sys_pipe,
        sys_read, sys_write,
    },
    *,
};

const USER_HEAP_SIZE: usize = 0x4000;
const USER_HEAP_GRANULARITY: usize = 8;

static mut USER_HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedBuddyHeap = LockedBuddyHeap::empty(USER_HEAP_GRANULARITY);

#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start(argc: usize, argv: usize) {
    unsafe {
        let start = USER_HEAP_SPACE.as_ptr() as usize;
        let end = start + USER_HEAP_SPACE.len();
        HEAP.add_segment(start, end);
    }

    let mut v = Vec::new();
    for i in 0..argc {
        let start =
            unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
        let len = (0usize..)
            .find(|i| unsafe { ((start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe { core::slice::from_raw_parts(start as *const u8, len) })
                .unwrap(),
        );
    }
    exit(main(argc, v.as_slice()));
}

#[linkage = "weak"]
#[no_mangle]
fn main(argc: usize, argv: &[&str]) -> i32 {
    panic!("[user] main() is not implemented.")
}

fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}

pub fn yield_now() {
    sys_yield();
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str, args: &Vec<*const u8>) -> isize {
    sys_exec(path, args)
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code) {
            -2 => yield_now(),
            pid => return pid,
        }
    }
}

pub fn waitpid(pid: isize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid, exit_code) {
            -2 => yield_now(),
            pid => return pid,
        }
    }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags)
}

pub fn close(fd: usize) {
    sys_close(fd);
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    sys_read(fd, buffer)
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn mkdir(dfd: usize, path: &str) -> isize {
    sys_mkdir(dfd, path)
}

pub fn chdir(path: &str) -> isize {
    sys_chdir(path)
}

pub fn getdents(dfd: usize, des: &[DirEntry]) -> isize {
    sys_getdents(dfd, des)
}

pub fn fstat(fd: usize, stat: &mut FileStat) {
    sys_fstat(fd, stat);
}

pub fn lseek(fd: usize, offset: usize, flag: SeekFlag) {
    sys_lseek(fd, offset, flag);
}

pub fn pipe(fds: &mut [usize; 2]) -> isize {
    sys_pipe(fds)
}
