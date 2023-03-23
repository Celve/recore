#![no_std]
#![no_main]
#![feature(linkage, panic_info_message)]

pub mod complement;
pub mod console;
pub mod syscall;

use allocator::heap::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 0x4000;
const USER_HEAP_GRANULARITY: usize = 8;

static mut USER_HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty(USER_HEAP_GRANULARITY);

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
    syscall_exit(exit_code);
}

pub fn yield_now() {
    syscall_yield();
}

pub fn fork() -> isize {
    syscall_fork()
}

pub fn exec(id: usize) -> isize {
    syscall_exec(id)
}
