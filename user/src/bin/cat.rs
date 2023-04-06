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
        println!("Usage: cat <filename>");
        return;
    }

    let filename = argv[1];
    let fd = open(filename, OpenFlags::RDONLY);
    if fd < 0 {
        println!("cat: {}: No such file or directory", filename);
        return;
    }

    let mut buf = [0u8; 1024];
    loop {
        let n = read(fd as usize, &mut buf);
        if n == 0 {
            break;
        }
        write(1, &buf[..n as usize]);
    }
}
