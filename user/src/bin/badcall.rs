#![no_main]
#![no_std]

use core::arch::asm;

extern crate user;

#[no_mangle]
fn main() {
    unsafe {
        asm!("mret");
    }
}
