use crate::{
    config::{
        MEMORY_END, PAGE_SIZE, TRAMPOLINE_START_ADDRESS, TRAP_CONTEXT_START_ADDRESS,
        UART_BASE_ADDRESS, UART_MAP_SIZE, USER_STACK_SIZE,
    },
    mm::address::PhyAddr,
    println,
    sync::up::UpCell,
    trap::trampoline::TRAMPOLINE,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use bitflags::bitflags;
use core::cmp::min;
use lazy_static::lazy_static;

use super::{
    address::{PhyPageNum, VirAddr, VirPageNum},
    frame::Area,
    page_table::PageTable,
    range::Range,
};

pub struct MemorySet {
    page_table: PageTable,
    areas: BTreeMap<VirPageNum, Arc<UpCell<Area>>>,
}

bitflags! {
    pub struct MappingPermission: usize {
        const R = 1 << 1; // Bit used to indicate readability.
        const W = 1 << 2; // Bit used to indicate writability.
        const X = 1 << 3; // Bit used to indicate whether executable.
        const U = 1 << 4; // Bit used to indicate user's accessibility.
    }
}

#[derive(PartialEq)]
pub enum MappingType {
    Identical,
    Framed,
    Linear,
}

impl MemorySet {
    pub fn empty() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: BTreeMap::new(),
        }
    }

    /// Kernel space should be initialized before page table is opened.
    pub fn new_kernel() -> Self {
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

        let mut result = Self::empty();

        // map .text section
        println!(
            "[kernel] Mapping .text section [{:#x}, {:#x})",
            stext as usize, etext as usize
        );
        result.map(
            VirAddr::from(stext as usize).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_identical(
                VirAddr::from(stext as usize).floor_to_vir_page_num(),
                VirAddr::from(etext as usize).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::X,
            ))),
        );

        // map .rodata section
        println!(
            "[kernel] Mapping .rodata section [{:#x}, {:#x})",
            srodata as usize, erodata as usize
        );
        result.map(
            VirAddr::from(srodata as usize).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_identical(
                VirAddr::from(srodata as usize).floor_to_vir_page_num(),
                VirAddr::from(erodata as usize).ceil_to_vir_page_num(),
                MappingPermission::R,
            ))),
        );

        // map .data section
        println!(
            "[kernel] Mapping .data section [{:#x}, {:#x})",
            sdata as usize, edata as usize
        );
        result.map(
            VirAddr::from(sdata as usize).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_identical(
                VirAddr::from(sdata as usize).floor_to_vir_page_num(),
                VirAddr::from(edata as usize).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::W,
            ))),
        );

        // map .bss section
        println!(
            "[kernel] Mapping .bss section [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        result.map(
            VirAddr::from(sbss_with_stack as usize).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_identical(
                VirAddr::from(sbss_with_stack as usize).floor_to_vir_page_num(),
                VirAddr::from(ebss as usize).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::W,
            ))),
        );

        println!(
            "[kernel] Mapping allocated section [{:#x}, {:#x})",
            ekernel as usize, MEMORY_END,
        );
        result.map(
            VirAddr::from(ekernel as usize).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_identical(
                VirAddr::from(ekernel as usize).floor_to_vir_page_num(),
                VirAddr::from(MEMORY_END).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::W,
            ))),
        );

        println!(
            "[kernel] Mapping memory-mapped registers for IO [{:#x}, {:#x})",
            UART_BASE_ADDRESS,
            UART_BASE_ADDRESS + UART_MAP_SIZE,
        );
        result.map(
            VirAddr::from(UART_BASE_ADDRESS).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_identical(
                VirAddr::from(UART_BASE_ADDRESS).floor_to_vir_page_num(),
                VirAddr::from(UART_BASE_ADDRESS + UART_MAP_SIZE).ceil_to_vir_page_num(),
                MappingPermission::R | MappingPermission::W,
            ))),
        );

        result.map_trampoline();

        result
    }

    pub fn from_elf(elf_data: &[u8]) -> Self {
        let mut result = Self::empty();

        // TODO: map trampoline
        let elf_file =
            xmas_elf::ElfFile::new(elf_data).expect("[memory_set] Fail to parse ELF file.");
        let elf_header = elf_file.header;
        let magic = elf_header.pt1.magic;
        if magic != [0x7f, 0x45, 0x4c, 0x46] {
            panic!("[memory_set] Invalid ELF file.");
        }

        let ph_count = elf_header.pt2.ph_count();
        let mut end_vpn = VirPageNum(0);
        for i in 0..ph_count {
            let ph = elf_file
                .program_header(i)
                .expect("[memory_set] Fail to get program header.");
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirAddr = (ph.virtual_addr() as usize + ph.mem_size() as usize).into();
                let mut map_perm = MappingPermission::U;
                let ph_flags = ph.flags();

                if ph_flags.is_read() {
                    map_perm |= MappingPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MappingPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MappingPermission::X;
                }
                let area = Area::new_identical(
                    start_va.floor_to_vir_page_num(),
                    end_va.ceil_to_vir_page_num(),
                    map_perm,
                );
                end_vpn = end_va.ceil_to_vir_page_num();
                area.copy_from(
                    &elf_file.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
                result.map(
                    start_va.floor_to_vir_page_num(),
                    Arc::new(UpCell::new(area)),
                );
            }
        }

        result.map_trampoline();

        result.map_trap_context();

        // map user stack
        let start_va: VirAddr = (end_vpn + 1).into();
        let end_va = start_va + USER_STACK_SIZE;
        result.map(
            start_va.floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_framed(
                end_va - start_va,
                MappingPermission::R | MappingPermission::W | MappingPermission::U,
            ))),
        );

        result
    }

    fn map(&mut self, vpn: VirPageNum, area: Arc<UpCell<Area>>) {
        let mut_area = area.borrow_mut();
        let range = Range::new(vpn, vpn + mut_area.len());
        range.iter().enumerate().for_each(|(i, vpn)| {
            self.page_table
                .map(vpn, mut_area.ppn(i), mut_area.map_perm().into())
        });
        drop(mut_area);

        self.areas.insert(vpn, area);
    }

    fn map_trampoline(&mut self) {
        self.map(
            VirAddr::from(TRAMPOLINE_START_ADDRESS).floor_to_vir_page_num(),
            TRAMPOLINE.clone(),
        );
    }

    fn map_trap_context(&mut self) {
        self.map(
            VirAddr::from(TRAP_CONTEXT_START_ADDRESS).floor_to_vir_page_num(),
            Arc::new(UpCell::new(Area::new_framed(
                1,
                MappingPermission::R | MappingPermission::W,
            ))),
        );
    }

    pub fn page_table(&self) -> &PageTable {
        &self.page_table
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: UpCell<MemorySet> = UpCell::new(MemorySet::new_kernel());
}
