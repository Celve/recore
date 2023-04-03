#![no_std]
#![no_main]

use core::mem::size_of;

use alloc::vec::Vec;
use fosix::fs::{DirEntry, FileStat, OpenFlags};
use user::{fstat, getdents, open};

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

#[no_mangle]
fn main() {
    let fd = open(".\0", OpenFlags::DIR);
    let mut stat = FileStat::empty();
    fstat(fd as usize, &mut stat);
    let size = stat.size();
    let len = size / size_of::<DirEntry>();

    let dents: Vec<DirEntry> = (0..len).map(|_| DirEntry::empty()).collect();
    getdents(fd as usize, &dents);

    println!("{}", len);
    dents.iter().for_each(|dent| {
        println!("{}", dent.name());
    });
}
