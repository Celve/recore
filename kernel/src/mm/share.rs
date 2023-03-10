use lazy_static::lazy_static;

use crate::{
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
        fn skernel();
        fn ekernel();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
    }

    let mut kernel_space = KERNEL_SPACE.borrow_mut();

    // map .text section
    kernel_space.push(
        VirAddr::from(skernel as usize).floor_to_vir_page_num(),
        VirAddr::from(ekernel as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::X,
    );

    // map .rodata section
    kernel_space.push(
        VirAddr::from(srodata as usize).floor_to_vir_page_num(),
        VirAddr::from(erodata as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R,
    );

    // map .data section
    kernel_space.push(
        VirAddr::from(sdata as usize).floor_to_vir_page_num(),
        VirAddr::from(edata as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );

    // map .bss section
    kernel_space.push(
        VirAddr::from(edata as usize).ceil_to_vir_page_num(),
        VirAddr::from(edata as usize).ceil_to_vir_page_num(),
        MappingType::Identical,
        PTEFlags::R | PTEFlags::W,
    );
}
