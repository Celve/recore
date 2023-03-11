#![no_std]
#![no_main]
#![feature(naked_functions, asm_const, const_cmp)]

#[macro_use]
extern crate alloc;

mod complement;
mod config;
mod heap;
mod io;
mod mm;
mod sync;

use config::*;
use core::arch::asm;
use heap::init_heap;
use io::uart::init_uart;

use crate::mm::{
    frame_allocator::init_frame_allocator,
    share::{activate_page_table, init_kernel_space},
};

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

unsafe fn rust_start() -> ! {
    use riscv::register::*;
    mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();
    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    asm!(
        "li t0, {medeleg}",
        "csrw medeleg, t0",
        "li t0, {mideleg}",
        "csrw mideleg, t0",
        "mret",
        medeleg = const 0xffff,
        mideleg = const 0xffff,
        options(noreturn),
    );
}

extern "C" fn rust_main() {
    init_bss();
    init_uart();
    println!("[kernel] Section bss cleared.");
    println!("[kernel] UART initialized.");

    init_heap();
    println!("[kernel] Heap initialized.");

    init_frame_allocator();
    println!("[kernel] Frame allocator initialized.");

    init_kernel_space();
    println!("[kernel] Kernel mapping done.");

    activate_page_table();
    println!("[kernel] Page table activated.");
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
