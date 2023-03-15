use crate::{
    config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE_START_ADDRESS, UART_BASE_ADDRESS, UART_MAP_SIZE},
    println,
    sync::up::UpCell,
};
use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use core::cmp::min;
use lazy_static::lazy_static;

use super::{
    address::{PhyPageNum, VirAddr, VirPageNum},
    frame_allocator::{alloc_series, Series},
    page_table::PageTable,
    range::Range,
};

pub struct MemorySet {
    page_table: Arc<UpCell<PageTable>>,
    mapping_areas: Vec<MappingArea>,
}

pub struct MappingArea {
    range: Range<VirPageNum>,
    page_table: Arc<UpCell<PageTable>>,
    series: Arc<UpCell<Series>>,
    mapping_type: MappingType,
    mapping_perm: MappingPermission,
}

bitflags! {
    pub struct MappingPermission: usize {
        const R = 1 << 1; // Bit used to indicate readability.
        const W = 1 << 2; // Bit used to indicate writability.
        const X = 1 << 3; // Bit used to indicate whether executable.
        const U = 1 << 4; // Bit used to indicate user's accessibility.
    }
}

pub enum MappingType {
    Identical,
    Framed,
    Linear,
}

impl MemorySet {
    pub fn empty() -> Self {
        Self {
            page_table: Arc::new(UpCell::new(PageTable::new())),
            mapping_areas: Vec::new(),
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

        let mut map_areas: Vec<MappingArea> = Vec::new();
        let page_table = Arc::new(UpCell::new(PageTable::new()));

        // map .text section
        println!(
            "[kernel] Mapping .text section [{:#x}, {:#x})",
            stext as usize, etext as usize
        );
        map_areas.push(MappingArea::new_identical(
            VirAddr::from(stext as usize).floor_to_vir_page_num(),
            VirAddr::from(etext as usize).ceil_to_vir_page_num(),
            page_table.clone(),
            MappingPermission::R | MappingPermission::X,
        ));

        // map .rodata section
        println!(
            "[kernel] Mapping .rodata section [{:#x}, {:#x})",
            srodata as usize, erodata as usize
        );
        map_areas.push(MappingArea::new_identical(
            VirAddr::from(srodata as usize).floor_to_vir_page_num(),
            VirAddr::from(erodata as usize).ceil_to_vir_page_num(),
            Arc::clone(&page_table),
            MappingPermission::R,
        ));

        // map .data section
        println!(
            "[kernel] Mapping .data section [{:#x}, {:#x})",
            sdata as usize, edata as usize
        );
        map_areas.push(MappingArea::new_identical(
            VirAddr::from(sdata as usize).floor_to_vir_page_num(),
            VirAddr::from(edata as usize).ceil_to_vir_page_num(),
            page_table.clone(),
            MappingPermission::R | MappingPermission::W,
        ));

        // map .bss section
        println!(
            "[kernel] Mapping .bss section [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        map_areas.push(MappingArea::new_identical(
            VirAddr::from(sbss_with_stack as usize).ceil_to_vir_page_num(),
            VirAddr::from(ebss as usize).ceil_to_vir_page_num(),
            page_table.clone(),
            MappingPermission::R | MappingPermission::W,
        ));

        println!(
            "[kernel] Mapping allocated section [{:#x}, {:#x})",
            ekernel as usize, MEMORY_END,
        );
        map_areas.push(MappingArea::new_identical(
            VirAddr::from(ekernel as usize).ceil_to_vir_page_num(),
            VirAddr::from(MEMORY_END).ceil_to_vir_page_num(),
            page_table.clone(),
            MappingPermission::R | MappingPermission::W,
        ));

        println!(
            "[kernel] Mapping memory-mapped registers for IO [{:#x}, {:#x})",
            UART_BASE_ADDRESS,
            UART_BASE_ADDRESS + UART_MAP_SIZE,
        );
        map_areas.push(MappingArea::new_identical(
            VirAddr::from(UART_BASE_ADDRESS).ceil_to_vir_page_num(),
            VirAddr::from(UART_BASE_ADDRESS + UART_MAP_SIZE).ceil_to_vir_page_num(),
            page_table.clone(),
            MappingPermission::R | MappingPermission::W,
        ));
        Self {
            page_table,
            mapping_areas: map_areas,
        }
    }

    pub fn from_elf(elf_data: &[u8]) -> Self {
        let page_table = Arc::new(UpCell::new(PageTable::new()));
        let mut mapping_areas: Vec<MappingArea> = Vec::new();

        // TODO: map trampoline
        let elf_file =
            xmas_elf::ElfFile::new(elf_data).expect("[memory_set] Fail to parse ELF file.");
        let elf_header = elf_file.header;
        let magic = elf_header.pt1.magic;
        if magic != [0x7f, 0x45, 0x4c, 0x46] {
            panic!("[memory_set] Invalid ELF file.");
        }

        let ph_count = elf_header.pt2.ph_count();
        let mut vpn_end = VirPageNum(0);
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
                let mut map_area = MappingArea::new_identical(
                    start_va.floor_to_vir_page_num(),
                    end_va.ceil_to_vir_page_num(),
                    Arc::clone(&page_table),
                    map_perm,
                );
                vpn_end = end_va.ceil_to_vir_page_num();
                map_area.copy_from(
                    &elf_file.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
                mapping_areas.push(map_area);
            }
        }
        Self {
            page_table,
            mapping_areas,
        }
    }

    pub fn map(
        &mut self,
        start: VirPageNum,
        end: VirPageNum,
        mapping_type: MappingType,
        mapping_perm: MappingPermission,
    ) {
        self.mapping_areas.push(match mapping_type {
            MappingType::Identical => {
                MappingArea::new_identical(start, end, Arc::clone(&self.page_table), mapping_perm)
            }
            MappingType::Framed => {
                MappingArea::new_identical(start, end, Arc::clone(&self.page_table), mapping_perm)
            }
            MappingType::Linear => {
                todo!()
                // MappingArea::new_identical(start, end, Arc::clone(&self.page_table), mapping_perm)
            }
        });
    }

    pub fn map_trampoline(&mut self) {
        self.mapping_areas.push(MappingArea::new_identical(
            TRAMPOLINE_START_ADDRESS.into(),
            (TRAMPOLINE_START_ADDRESS + PAGE_SIZE).into(),
            Arc::clone(&self.page_table),
            MappingPermission::R | MappingPermission::X,
        ));
    }

    pub fn unmap(&mut self, start: VirPageNum) -> bool {
        let pos = self
            .mapping_areas
            .iter()
            .position(|area| area.range.start == start);

        match pos {
            Some(pos) => {
                self.mapping_areas.remove(pos);
                true
            }
            None => false,
        }
    }

    pub fn page_table(&self) -> &Arc<UpCell<PageTable>> {
        &self.page_table
    }
}

impl MappingArea {
    pub fn new_identical(
        start: VirPageNum,
        end: VirPageNum,
        page_table: Arc<UpCell<PageTable>>,
        mapping_perm: MappingPermission,
    ) -> Self {
        let range = Range::new(start, end);
        let mut result = Self {
            range,
            page_table,
            series: Arc::new(UpCell::new(Series::new(
                range.iter().map(|vpn| PhyPageNum::from(vpn)).collect(),
                false,
            ))),
            mapping_type: MappingType::Identical,
            mapping_perm,
        };
        result.map();
        result
    }

    pub fn new_framed(
        start: VirPageNum,
        end: VirPageNum,
        page_table: Arc<UpCell<PageTable>>,
        mapping_perm: MappingPermission,
    ) -> Self {
        let mut result = Self {
            range: Range::new(start, end),
            page_table,
            series: Arc::new(UpCell::new(alloc_series(end - start))),
            mapping_type: MappingType::Framed,
            mapping_perm,
        };
        result.map();
        result
    }

    pub fn new_linear(start: VirPageNum, series: Arc<UpCell<Series>>) -> Self {
        let len = series.borrow_mut().len();
        let mut result = Self {
            range: Range::new(start, start + len),
            page_table: Arc::new(UpCell::new(PageTable::new())),
            series,
            mapping_type: MappingType::Linear,
            mapping_perm: MappingPermission::R | MappingPermission::W,
        };
        result.map();
        result
    }

    pub fn copy_from(&mut self, data: &[u8]) {
        let series = self.series.borrow_mut();
        let mut start = 0;
        let len = data.len();
        for ppn in series.iter() {
            let src = &data[start..min(len, start + PAGE_SIZE)];
            let dst = ppn.as_raw_bytes();
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
        }
    }

    pub fn map(&mut self) {
        let mut page_table = self.page_table.borrow_mut();
        let series = self.series.borrow_mut();
        self.range
            .iter()
            .enumerate()
            .for_each(|(i, vpn)| page_table.map(vpn, series.ppn(i), self.mapping_perm.into()));
    }

    pub fn unmap(&mut self) {
        let mut page_table = self.page_table.borrow_mut();
        self.range.iter().for_each(|vpn| page_table.unmap(vpn));
    }
}

impl Drop for MappingArea {
    fn drop(&mut self) {
        self.unmap();
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: UpCell<MemorySet> = UpCell::new(MemorySet::empty());
}
