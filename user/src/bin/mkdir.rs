#![no_std]
#![no_main]

use core::mem::size_of;

use alloc::{string::String, vec::Vec};
use fosix::fs::{DirEntry, FileStat, OpenFlags};
use user::{fstat, getdents, mkdir, open};

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) {
    if argc == 1 {
        println!("[user] cd: missing operand");
    } else {
        let mut path = String::from(argv[1]);
        if path.ends_with("/") {
            path.pop();
        }
        path.push('\0');
        let dfd = open(".\0", OpenFlags::DIR);
        assert_ne!(dfd, -1);
        if mkdir(dfd as usize, path.as_str()) == -1 {
            println!("[user] mkdir {}: Fail to make such directory", argv[1]);
        } else {
            println!("");
        }
    }
}
