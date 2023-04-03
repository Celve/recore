#![no_main]
#![no_std]

#[macro_use]
extern crate user;

use fosix::fs::OpenFlags;
use user::{exec, fork, open, wait, yield_now};

#[no_mangle]
fn main() {
    let f = open("fantastic", OpenFlags::RDWR | OpenFlags::CREATE);
    if f < 0 {
        println!("open failed");
        return;
    }
}
