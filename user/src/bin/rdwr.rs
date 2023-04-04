#![no_main]
#![no_std]

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

use alloc::vec;
use fosix::fs::{FileStat, OpenFlags, SeekFlag};
use user::{exec, fork, fstat, lseek, open, read, wait, write, yield_now};

#[no_mangle]
fn main() {
    let fd = open("fantastic\0", OpenFlags::RDWR | OpenFlags::CREATE);
    if fd < 0 {
        println!("open failed");
        return;
    }
    write(fd as usize, "hello world\n\0".as_bytes());

    let mut stat = FileStat::empty();
    fstat(fd as usize, &mut stat);
    let size = stat.size();
    let mut buf = vec![0u8; size as usize];

    lseek(fd as usize, 0, SeekFlag::SET);
    read(fd as usize, &mut buf);
    buf.iter().for_each(|c| print!("{}", *c as char));
}
