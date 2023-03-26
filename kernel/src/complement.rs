use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    println!("[kernel] {}", panic_info);
    loop {}
}
