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
    let sig = argv[1][1..].parse::<usize>().unwrap();
    let pid = argv[2].parse::<usize>().unwrap();
    println!("kill process {} with {}.", pid, sig);
    kill(pid, sig);
}
