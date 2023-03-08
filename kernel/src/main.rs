#![no_std]
#![no_main]
#![feature(naked_functions, asm_const, const_cmp)]

extern crate alloc;

mod complement;
mod config;
mod heap;
mod io;
mod sync;

use alloc::boxed::Box;
use config::*;
use core::arch::asm;
use heap::HEAP_ALLOCATOR;
use io::uart::UART;

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
        "j {rust_main}",
        bootloader_stack = sym BOOTLOADER_STACK_SPACE,
        bootloader_stack_size = const BOOTLOADER_STACK_SIZE,
        rust_main = sym rust_main,
        options(noreturn),
    );
}

#[no_mangle]
extern "C" fn rust_main() {
    UART.init();
    HEAP_ALLOCATOR.init();
    let a = Box::new(1);
    println!("Hello, world!");
    println!("This is the {} message", a);
}
