#![no_std]
#![no_main]

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

use alloc::string::String;
use user::{console, exec, fork, waitpid};

const BS: char = 8 as char;
const DL: char = 127 as char;

fn getline() -> String {
    let mut c = console::stdin().getchar();
    let mut result = String::new();
    while c != '\n' && c != '\r' {
        if c == BS || c == DL {
            if !result.is_empty() {
                print!("{c}");
                result.pop();
            }
        } else {
            result.push(c);
            print!("{c}");
        }
        c = console::stdin().getchar();
    }
    result
}

#[no_mangle]
fn main() {
    loop {
        print!("> ");
        let str = getline();
        if str.is_empty() {
            println!("");
            continue;
        }
        let pid = fork();
        if pid == 0 {
            if exec(str.as_str()) == -1 {
                println!("[user] Exec {} failed", str);
                return;
            }
        } else {
            let mut exit_code: i32 = 0;
            waitpid(pid, &mut exit_code);
            println!("[user] Process {} exit with code {}", pid, exit_code);
        }
    }
}
