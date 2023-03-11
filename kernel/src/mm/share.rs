use crate::println;
use core::arch::asm;
use lazy_static::lazy_static;

use crate::{
    config::{MEMORY_END, UART_BASE_ADDRESS, UART_MAP_LENGTH},
    mm::{address::VirAddr, memory::MappingType, page_table::PTEFlags},
    sync::up::UpCell,
};

use super::memory::MemorySet;

lazy_static! {
    static ref KERNEL_SPACE: UpCell<MemorySet> = UpCell::new(MemorySet::new());
}

/// Kernel space should be initialized before page table is opened.
#[no_mangle]
pub fn init_kernel_space() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss_with_stack();
        fn ebss();
        fn ekernel();
    }

    let mut kernel_space = KERNEL_SPACE.borrow_mut();

    // map .text section
    println!(
        "[kernel] Mapping .text section [{:#x}, {:#x})",
        stext as usize, etext as usize
    );
    kernel_space.push(
        VirAddr::from(stext as usize).floor_to_vir_page_num(),
        VirAddr::from(etext as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::X,
    );

    // map .rodata section
    println!(
        "[kernel] Mapping .rodata section [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    kernel_space.push(
        VirAddr::from(srodata as usize).floor_to_vir_page_num(),
        VirAddr::from(erodata as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R,
    );

    // map .data section
    println!(
        "[kernel] Mapping .data section [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    kernel_space.push(
        VirAddr::from(sdata as usize).floor_to_vir_page_num(),
        VirAddr::from(edata as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );

    // map .bss section
    println!(
        "[kernel] Mapping .bss section [{:#x}, {:#x})",
        sbss_with_stack as usize, ebss as usize
    );
    kernel_space.push(
        VirAddr::from(sbss_with_stack as usize).ceil_to_vir_page_num(),
        VirAddr::from(ebss as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );

    println!(
        "[kernel] Mapping allocated section [{:#x}, {:#x})",
        ekernel as usize, MEMORY_END,
    );
    kernel_space.push(
        VirAddr::from(ekernel as usize).ceil_to_vir_page_num(),
        VirAddr::from(MEMORY_END).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );

    println!(
        "[kernel] Mapping memory-mapped registers for IO [{:#x}, {:#x})",
        UART_BASE_ADDRESS,
        UART_BASE_ADDRESS + UART_MAP_LENGTH,
    );
    kernel_space.push(
        VirAddr::from(UART_BASE_ADDRESS).ceil_to_vir_page_num(),
        VirAddr::from(UART_BASE_ADDRESS + UART_MAP_LENGTH).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );
}

pub fn activate_page_table() {
    let kernel_space = KERNEL_SPACE.borrow_mut();
    let page_table = kernel_space.page_table().borrow_mut();
    let satp = page_table.to_satp();
    riscv::register::satp::write(satp);
    unsafe {
        asm!("sfence.vma");
    }
}
