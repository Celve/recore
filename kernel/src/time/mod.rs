use core::arch::global_asm;

use crate::{
    config::{CLINT, CPUS, SCHED_PERIOD},
    task::processor::hart_id,
};
use riscv::register::*;

global_asm!(include_str!("trap.s"));

pub fn set_timer(id: usize, time: usize) {
    unsafe {
        let timer = (CLINT + 0x4000 + 8 * id) as *mut usize;
        *timer = time;
    }
}

pub fn get_time() -> usize {
    unsafe {
        let time = (CLINT + 0xbff8) as *const usize;
        *time
    }
}

#[link_section = ".bss.stack"]
#[no_mangle]
pub static mut TIMER_SCRATCH: [[usize; 5]; CPUS] = [[0; 5]; CPUS];

#[no_mangle]
pub unsafe fn init_timer() {
    let id = hart_id();

    // setup timer
    set_timer(id, get_time() + SCHED_PERIOD);

    // prepare information in scratch[] for timervec
    // scratch[0..2] : space for timervec to save registers
    // scratch[3] : address of CLINT MTIMECMP register
    // scratch[4] : desired interval (in cycles) between timer interrupts
    let scratch = &mut TIMER_SCRATCH[id];
    scratch[3] = CLINT + 0x4000 + 8 * id;
    scratch[4] = SCHED_PERIOD;

    mscratch::write(scratch as *mut usize as usize);

    // set the machine-mode trap handler
    extern "C" {
        fn _timertrap();
    }
    mtvec::write(_timertrap as usize, mtvec::TrapMode::Direct);

    // enable machine-mode interrupts
    mstatus::set_mie();

    // enable machine-mode timer interrupts
    mie::set_mtimer();
}

/// It is spinning until time is up.
pub fn sleep(interval: usize) {
    let limit = get_time() + interval;
    while get_time() < limit {}
}
