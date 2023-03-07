use core::panic::PanicInfo;

use crate::println;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    println!("[kernel] Panick!");
    loop {}
}
