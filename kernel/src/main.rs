#![no_std]
#![no_main]
#![feature(
    naked_functions,
    asm_const,
    const_cmp,
    fn_align,
    inline_const,
    maybe_uninit_uninit_array,
    const_maybe_uninit_uninit_array,
    const_trait_impl,
    const_for,
    const_mut_refs,
    sync_unsafe_cell
)]

#[macro_use]
extern crate alloc;

#[macro_use]
mod io;

mod complement;
mod config;
mod drivers;
mod fs;
mod ipc;
mod mem;
mod mm;
mod proc;
mod sync;
mod syscall;
mod task;
mod time;
mod trap;

use config::*;
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, Ordering},
};
use drivers::{
    plic::{TargetPriority, PLIC},
    uart::UART,
};
use mm::page_table::activate_page_table;
use proc::manager::PROC_MANAGER;
use riscv::register::*;
use task::processor::{Processor, PROCESSORS};
use time::init_timer;

use crate::{
    fs::FS,
    mem::{normal::init_frame_allocator, slab::init_slab},
    time::get_time,
    trap::set_kernel_stvec,
};

#[link_section = ".bss.stack"]
static mut BOOTLOADER_STACK_SPACE: [[u8; BOOTLOADER_STACK_SIZE]; CPUS] =
    [[0; BOOTLOADER_STACK_SIZE]; CPUS];

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() {
    asm!(
        "la sp, {bootloader_stack}",
        "li t0, {bootloader_stack_size}",
        "csrr t1, mhartid",
        "addi t1, t1, 1",
        "mul t0, t0, t1",
        "add sp, sp, t0",
        "j {rust_start}",
        bootloader_stack = sym BOOTLOADER_STACK_SPACE,
        bootloader_stack_size = const BOOTLOADER_STACK_SIZE,
        rust_start = sym rust_start,
        options(noreturn),
    );
}

/// An entry for rust program that transfers state from mmode to smode.
///
/// The stack it used is the bootloader stack, which is still in used afterwards for scheduler.
unsafe fn rust_start() -> ! {
    // mstatus set for privilege change, mepc set for correct jumping
    mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    // disable page table for the supervisor mode
    satp::write(0);

    // the following two lines are necessary, but I don't know why
    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    // keep CPU's hartid in tp register
    asm!("csrr tp, mhartid");

    init_timer();

    asm!(
        "csrw mideleg, {mideleg}", // some bits could not be set by this method
        "csrw medeleg, {medeleg}",
        "mret",
        medeleg = in(reg) !0,
        mideleg = in(reg) !0,
        options(noreturn),
    );
}

static INITED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
extern "C" fn rust_main() {
    if Processor::hart_id() == 0 {
        set_kernel_stvec();
        init_trap();
        init_bss();
        init_uart();
        infoln!("Cleared .bss section.");
        infoln!("Initialized UART.");

        init_slab();
        infoln!("Initialized buddy allocator and slab allocator.");

        init_frame_allocator();
        infoln!("Initialized frame allocator.");

        activate_page_table(); // the kernel space is automatically init before activating page table because of the lazy_static!
        infoln!("Activated page table with satp {}.", satp::read().bits());

        init_devices();
        init_tasks();

        infoln!(
            "Hart {} begins to run tasks in {}.",
            Processor::hart_id(),
            get_time()
        );
        INITED.store(true, Ordering::Release);
    } else {
        while !INITED.load(Ordering::Acquire) {}
        set_kernel_stvec();
        init_trap();
        activate_page_table();
        init_devices();
        infoln!(
            "Hart {} begins to run tasks in {}.",
            Processor::hart_id(),
            get_time()
        );
    }
    Processor::run_tasks();
}

fn init_trap() {
    unsafe {
        sie::set_sext();
        sie::set_stimer();
        sie::set_ssoft();
    }
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

fn init_uart() {
    UART.init();
}

fn init_devices() {
    let hart_id = Processor::hart_id();

    // set the threshold for each target respectively, to disable notifications for machine mode
    PLIC.set_threshold(hart_id, TargetPriority::Machine, 1);
    PLIC.set_threshold(hart_id, TargetPriority::Supervisor, 0);

    // currently, only notifications from uart are enabled
    // 1 stands for block, and 10 stands for uart
    // set priority and enable the interrupt for each src
    for src_id in [1, 10] {
        PLIC.set_priority(src_id, 1);
        PLIC.enable(hart_id, TargetPriority::Supervisor, src_id);
    }

    // enable external interrupt for supervisor mode
    unsafe {
        sie::set_sext();
    }
}

fn init_tasks() {
    let task = PROC_MANAGER.get(1).unwrap().lock().main_task();
    PROCESSORS[Processor::hart_id()].lock().push_normal(&task);
    FS.disk_manager().enable_non_blocking();
}
