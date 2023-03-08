use core::mem::size_of;

pub const BOOTLOADER_STACK_SIZE: usize = 0x4096;
pub const UART_BASE_ADDRESS: usize = 0x10_000_000;
pub const KERNEL_HEAP_GRANULARITY: usize = size_of::<usize>();
pub const KERNEL_HEAP_SIZE: usize = 0x200_000;
