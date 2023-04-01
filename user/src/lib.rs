#![no_std]
#![no_main]
#![feature(linkage, panic_info_message)]

extern crate alloc;

pub mod complement;
pub mod console;
pub mod syscall;

use allocator::heap::LockedBuddyHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 0x4000;
const USER_HEAP_GRANULARITY: usize = 8;

static mut USER_HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedBuddyHeap = LockedBuddyHeap::empty(USER_HEAP_GRANULARITY);

#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        let start = USER_HEAP_SPACE.as_ptr() as usize;
        let end = start + USER_HEAP_SPACE.len();
        HEAP.add_segment(start, end);
    }
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
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

pub fn exec(path: &str) -> isize {
    sys_exec(path)
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
