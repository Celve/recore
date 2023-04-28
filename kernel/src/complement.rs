use core::panic::PanicInfo;

#[no_mangle]
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    fatalln!("Panicked at {}", panic_info);
    loop {}
}
