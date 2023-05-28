#![no_main]
#![no_std]

#[macro_use]
extern crate user;

static mut FND: [u8; 4096] = [0; 4096];
static mut FND2: [u8; 4096] = [0; 4096];

#[no_mangle]
fn main() {
    #[allow(unused)]
    let stack0 = [0u8; 10];
    let mut stack = [0u8; 2050];
    let mut stack2 = [0u8; 10];
    for i in 0..2050 {
        unsafe {
            FND[i] = (i % 255) as u8;
        }
    }
    stack[2049] = 1;
    for i in 0..2050 {
        unsafe {
            stack[i] = FND[i];
        }
    }
    println!("{}", unsafe { FND[65] });
    for i in 0..2050 {
        unsafe {
            FND2[i] = (i % 255) as u8;
        }
    }
    stack[2049] = 1;
    for i in 0..2050 {
        unsafe {
            stack[i] = FND2[i];
        }
    }
    unsafe {
        stack2[0] = FND[66];
        stack2[1] = FND[69];
    }
    unsafe {
        println!("{}", FND2[25]);
        println!(
            "{}",
            (((stack.as_ptr() as usize) + 2050) as *const u8).read_volatile()
        );
        println!("{}", *(((stack.as_ptr() as usize) - 1) as *const u8));
    }
}
