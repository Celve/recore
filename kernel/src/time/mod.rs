use core::arch::global_asm;

use crate::config::{CLINT, NCPU, TIMER_INTERVAL};
use lazy_static::lazy_static;
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
pub static mut TIMER_SCRATCH: [[usize; 5]; NCPU] = [[0; 5]; NCPU];

#[no_mangle]
pub unsafe fn init_timer() {
    let id = mhartid::read();

    // setup timer
    set_timer(id, get_time() + TIMER_INTERVAL);

    // prepare information in scratch[] for timervec
    // scratch[0..2] : space for timervec to save registers
    // scratch[3] : address of CLINT MTIMECMP register
    // scratch[4] : desired interval (in cycles) between timer interrupts
    let scratch = &mut TIMER_SCRATCH[id];
    scratch[3] = CLINT + 0x4000 + 8 * id;
    scratch[4] = TIMER_INTERVAL;
    // *scratch.add(3) = CLINT + 0x4000 + 8 * id;
    // *scratch.add(4) = TIMER_INTERVAL;

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
