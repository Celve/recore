#![no_std]
#![no_main]

#[macro_use]
extern crate user;

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use fosix::fs::OpenFlags;
use user::{chdir, console, exec, fork, mkdir, open, waitpid};

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
    let mut cwd = String::from("/");
    loop {
        print!("{} > ", cwd);
        let str = getline();
        if str.is_empty() {
            println!("");
            continue;
        }

        let args: Vec<&str> = str.split_whitespace().collect();

        match args[0] {
            "cd" => {
                if args.len() == 1 {
                    println!("[user] cd: missing operand");
                } else {
                    let mut path = String::from(args[1]);
                    if path.ends_with("/") {
                        path.pop();
                    }
                    path.push('\0');
                    if chdir(path.as_str()) == -1 {
                        println!("[user] cd {}: No such file or directory", args[1]);
                    } else if args[1] == ".." && cwd != "/" {
                        cwd.pop();
                        let mut c = cwd.pop();
                        while c != Some('/') {
                            c = cwd.pop();
                        }
                        cwd.push('/');
                    } else if args[1] != ".." {
                        cwd.push_str(args[1]);
                        cwd.push('/');
                    }
                    println!("");
                }
            }

            "mkdir" => {
                if args.len() == 1 {
                    println!("[user] cd: missing operand");
                } else {
                    let mut path = String::from(args[1]);
                    if path.ends_with("/") {
                        path.pop();
                    }
                    path.push('\0');
                    let dfd = open(".\0", OpenFlags::DIR);
                    assert_ne!(dfd, -1);
                    if mkdir(dfd as usize, path.as_str()) == -1 {
                        println!("[user] mkdir {}: Fail to make such directory", args[1]);
                    } else {
                        println!("");
                    }
                }
            }

            _ => {
                let mut name = String::from(args[0]);
                name.push('\0');
                let pid = fork();
                if pid == 0 {
                    if exec(name.as_str()) == -1 {
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
    }
}
