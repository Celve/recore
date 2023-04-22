use core::mem::size_of;

pub const BOOTLOADER_STACK_SIZE: usize = 0x10000;
pub const UART_BASE_ADDRESS: usize = 0x10_000_000;
pub const UART_MAP_SIZE: usize = 0x6;

// pub const KERNEL_HEAP_GRANULARITY: usize = size_of::<usize>();
pub const KERNEL_HEAP_GRANULARITY: usize = PAGE_SIZE;
pub const KERNEL_HEAP_SIZE: usize = 0x200_000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const PA_WIDTH: usize = 56;
pub const VA_WIDTH: usize = 39;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;
pub const PTE_FLAG_WIDTH: usize = 10;

pub const MEMORY_END: usize = 0x88_000_000;

pub const TRAMPOLINE_ADDR: usize = usize::MAX - PAGE_SIZE + 1; // `usize::MAX` is included.

pub const USER_STACK_SIZE: usize = 0x10000;
pub const KERNEL_STACK_SIZE: usize = 0x10000;

pub const TIMER_INTERVAL: usize = 100_000;
pub const CLINT: usize = 0x2000000;

pub const CACHE_SIZE: usize = 512;
pub const BLK_SIZE: usize = 512;
pub const INODE_SIZE: usize = 32 * 4;
pub const DNODE_SIZE: usize = 32 * 16;
pub const INODE_PER_BLK: usize = BLK_SIZE / INODE_SIZE;
pub const DIR_ENTRY_NAME_LEN: usize = 28;
pub const FUSE_INODE_NUM: usize = 1024;
pub const FUSE_DNODE_NUM: usize = 4096;

pub const VIRTIO_ADDR: usize = 0x10_000_000;
pub const VIRTIO_SIZE: usize = 0x9000;

pub const RING_BUFFER_SIZE: usize = 128;

pub const NUM_SIGNAL: usize = 32;

pub const CLOCK_FREQ: usize = 12500000;

pub const VIRT_PLIC_ADDR: usize = 0xc00_0000;
pub const VIRT_PLIC_SIZE: usize = 0x210_000;
pub const VIRT_UART: usize = 0x10_000_000;

pub const CPUS: usize = 4;
