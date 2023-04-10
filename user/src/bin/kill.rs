#![no_std]
#![no_main]

use user::kill;

#[macro_use]
extern crate user;

extern crate alloc;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) {
    assert!(argc == 3);
    assert!(argv[1].starts_with("-"));
    let sig = argv[1][1..argv[1].len() - 1].parse::<usize>().unwrap();
    let pid = argv[2][0..argv[2].len() - 1].parse::<usize>().unwrap();
    kill(pid, sig);
}
