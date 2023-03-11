use lazy_static::lazy_static;

use crate::{
    mm::{address::VirAddr, memory::MappingType, page_table::PTEFlags},
    println,
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
        fn skernel();
        fn ekernel();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
    }

    let mut kernel_space = KERNEL_SPACE.borrow_mut();

    // map .text section
    println!(
        "[kernel] Mapping .text section [{:#x}, {:#x})",
        skernel as usize, ekernel as usize
    );
    kernel_space.push(
        VirAddr::from(skernel as usize).floor_to_vir_page_num(),
        VirAddr::from(ekernel as usize).ceil_to_vir_page_num(),
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
        sbss as usize, ebss as usize
    );
    kernel_space.push(
        VirAddr::from(sbss as usize).ceil_to_vir_page_num(),
        VirAddr::from(ebss as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );
}
