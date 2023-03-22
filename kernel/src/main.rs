#![no_std]
#![no_main]
#![feature(naked_functions, asm_const, const_cmp)]

#[macro_use]
extern crate alloc;

#[macro_use]
mod io;

mod complement;
mod config;
mod mm;
mod syscall;
mod task;
mod trap;

use config::*;
use core::arch::{asm, global_asm};
use io::uart::init_uart;
use mm::{frame::init_frame_allocator, heap::init_heap, page_table::activate_page_table};

use crate::task::manager::run_tasks;

global_asm!(include_str!("app.s"));

#[link_section = ".bss.stack"]
static mut BOOTLOADER_STACK_SPACE: [u8; BOOTLOADER_STACK_SIZE] = [0; BOOTLOADER_STACK_SIZE];

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() {
    asm!(
        "la sp, {bootloader_stack}",
        "li t0, {bootloader_stack_size}",
        "add sp, sp, t0",
        "j {rust_start}",
        bootloader_stack = sym BOOTLOADER_STACK_SPACE,
        bootloader_stack_size = const BOOTLOADER_STACK_SIZE,
        rust_start = sym rust_start,
        options(noreturn),
    );
}

/// An entry for rust program that transfers mmode to smode.
///
/// We spare a little bit of kernel stack to use as its stack, which would never be recovered.
unsafe fn rust_start() -> ! {
    use riscv::register::*;
    mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();

    // the following two lines are necessary, but I don't know why
    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    asm!(
        "li t0, {medeleg}",
        "li t1, {mideleg}",
        "csrw medeleg, t0",
        "csrw mideleg, t1",
        "mret",
        medeleg = const 0xffff,
        mideleg = const 0xffff,
        options(noreturn),
    );
}

#[no_mangle]
extern "C" fn rust_main() {
    init_bss();
    init_uart();
    println!("[kernel] Section bss cleared.");
    println!("[kernel] UART initialized.");

    init_heap();
    println!("[kernel] Heap initialized.");

    init_frame_allocator();
    println!("[kernel] Frame allocator initialized.");

    activate_page_table(); // the kernel space is automatically init before activating page table because of the lazy_static!
    println!("[kernel] Page table activated.");

    println!("[kernel] Begin to run kernel tasks.");
    run_tasks();
}

fn init_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as *mut u8, ebss as usize - sbss as usize).fill(0);
    }
}
