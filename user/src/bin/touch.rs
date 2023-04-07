#![no_std]
#![no_main]

use fosix::fs::OpenFlags;
use user::{open, read, write};

#[macro_use]
extern crate user;

extern crate alloc;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) {
    if argc != 2 {
        println!("Usage: touch <filename>");
        return;
    }

    let filename = argv[1];
    let fd = open(filename, OpenFlags::CREATE);
    if fd < 0 {
        println!("touch: fail to create {}", filename);
    }
}
