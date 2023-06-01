use crate::io::log::LogLevel;

pub const BOOTLOADER_STACK_SIZE: usize = 0x10000;
pub const UART_BASE_ADDRESS: usize = 0x10_000_000;
pub const UART_MAP_SIZE: usize = 0x6;

// pub const KERNEL_HEAP_GRANULARITY: usize = size_of::<usize>();
pub const KERNEL_HEAP_GRANULARITY: usize = PAGE_SIZE;
pub const KERNEL_HEAP_SIZE: usize = 0x800_000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const PA_WIDTH: usize = 56;
pub const VA_WIDTH: usize = 39;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;
pub const PTE_FLAG_WIDTH: usize = 10;

pub const MEMORY_END: usize = 0x88_000_000;
pub const KERNEL_START: usize = 0x8000_0000;
pub const KERNEL_SIZE: usize = MEMORY_END - KERNEL_START;
pub const KERNEL_PAGE_NUM: usize = KERNEL_SIZE / PAGE_SIZE;

pub const TRAMPOLINE_ADDR: usize = usize::MAX - PAGE_SIZE + 1; // `usize::MAX` is included.

pub const USER_STACK_SIZE: usize = 0x10000;
pub const KERNEL_STACK_SIZE: usize = 0x10000;

pub const SCHED_PERIOD: usize = 1_000_000;
pub const PELT_PERIOD: usize = 2 * SCHED_PERIOD; // the only requirement is that it should be larger than SCHED_PERIOD
pub const PELT_ATTENUATION: usize = 10;
pub const MIN_AVG_TIME_SLICE: usize = SCHED_PERIOD / 8;
pub const MIN_EXEC_TIME_SLICE: usize = SCHED_PERIOD / 1000;
pub const CLINT: usize = 0x2000000;

pub const VIRTIO_ADDR: usize = 0x10_000_000;
pub const VIRTIO_SIZE: usize = 0x9000;

pub const RING_BUFFER_SIZE: usize = 128;

pub const NUM_SIGNAL: usize = 32;

pub const CLOCK_FREQ: usize = 12500000;

pub const VIRT_PLIC_ADDR: usize = 0xc00_0000;
pub const VIRT_PLIC_SIZE: usize = 0x210_000;
pub const VIRT_UART: usize = 0x10_000_000;
pub const VIRT_IO_HEADER: usize = 0x1000_1000;
pub const VIRT_TEST: usize = 0x100000;
pub const VIRT_TEST_SIZE: usize = 0x2000;

pub const CPUS: usize = 4;

pub const LOG_LEVEL: LogLevel = LogLevel::Debug;
