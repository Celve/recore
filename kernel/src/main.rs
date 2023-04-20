#![no_std]
#![no_main]
#![feature(naked_functions, asm_const, const_cmp, fn_align)]

#[macro_use]
extern crate alloc;

#[macro_use]
mod io;

mod complement;
mod config;
mod drivers;
mod fs;
mod heap;
mod ipc;
mod mm;
mod proc;
mod sync;
mod syscall;
mod task;
mod time;
mod trap;

use config::*;
use core::arch::{asm, global_asm};
use drivers::plic::{TargetPriority, PLIC};
use heap::init_heap;
use io::uart::init_uart;
use mm::{frame::init_frame_allocator, page_table::activate_page_table};
use riscv::register::*;
use task::processor::run_tasks;
use time::init_timer;

use crate::{fs::FUSE, trap::set_kernel_stvec};

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
    mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);

    // the following two lines are necessary, but I don't know why
    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    // keep CPU's hartid in tp register
    asm!("csrr tp, mhartid");

    init_timer();

    mideleg::set_stimer();
    mideleg::set_sext();
    mideleg::set_ssoft();

    asm!(
        "csrw mideleg, {mideleg}", // some bits could not be set by this method
        "csrw medeleg, {medeleg}",
        "mret",
        medeleg = in(reg) !0,
        mideleg = in(reg) !0,
        options(noreturn),
    );
}

#[no_mangle]
extern "C" fn rust_main() {
    unsafe {
        sie::set_sext();
        sie::set_stimer();
        sie::set_ssoft();
    }

    set_kernel_stvec();

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

    let root = FUSE.root();
    root.lock()
        .ls()
        .iter()
        .for_each(|name| println!("{}", name));

    // init_devices();

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

fn init_devices() {
    let hart_id = 0; // TODO: this should be fixed when SMP is enabled

    // set the threshold for each target respectively, to disable notifications for machine mode
    PLIC.set_threshold(hart_id, TargetPriority::Machine, 1);
    PLIC.set_threshold(hart_id, TargetPriority::Supervisor, 0);

    // 8 stands for block, and 10 stands for uart
    // set priority and enable the interrupt for each src
    for src_id in [8, 10] {
        PLIC.set_priority(src_id, 1);
        PLIC.enable(hart_id, TargetPriority::Supervisor, src_id);
    }

    // enable external interrupt for supervisor mode
    unsafe {
        sie::set_sext();
    }
}
