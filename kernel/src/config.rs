use core::mem::size_of;

pub const BOOTLOADER_STACK_SIZE: usize = 0x4096;
pub const UART_BASE_ADDRESS: usize = 0x10_000_000;
pub const UART_MAP_LENGTH: usize = 0x6;

pub const KERNEL_HEAP_GRANULARITY: usize = size_of::<usize>();
// pub const KERNEL_HEAP_GRANULARITY: usize = PAGE_SIZE;
pub const KERNEL_HEAP_SIZE: usize = 0x200_000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const PA_WIDTH: usize = 56;
pub const VA_WIDTH: usize = 39;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;
pub const PTE_FLAG_WIDTH: usize = 10;

pub const MEMORY_END: usize = 0x88_000_000;
