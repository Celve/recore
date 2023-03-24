use super::{
    address::{VirAddr, VirPageNum},
    area::Area,
    page_table::PageTable,
};

use crate::{
    config::{
        MEMORY_END, PAGE_SIZE, TRAMPOLINE_START_ADDRESS, TRAP_CONTEXT_END_ADDRESS,
        TRAP_CONTEXT_START_ADDRESS, UART_BASE_ADDRESS, UART_MAP_SIZE, USER_STACK_SIZE,
    },
    mm::{address::PhyAddr, page_table::PTEFlags},
    println,
};

use alloc::vec::Vec;
use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

pub struct Memory {
    page_table: PageTable,
    pub areas: Vec<Area>,
}

bitflags! {
    pub struct MappingPermission: u8 {
        const R = 1 << 1; // Bit used to indicate readability.
        const W = 1 << 2; // Bit used to indicate writability.
        const X = 1 << 3; // Bit used to indicate whether executable.
        const U = 1 << 4; // Bit used to indicate user's accessibility.
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MappingType {
    Identical,
    Framed,
    Linear,
}

impl Memory {
    pub fn empty() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
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
        result.map(Area::new_identical(
            VirAddr::from(stext as usize).floor_to_vir_page_num(),
            VirAddr::from(etext as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::X,
        ));

        // map .rodata section
        println!(
            "[kernel] Mapping .rodata section [{:#x}, {:#x})",
            srodata as usize, erodata as usize
        );
        result.map(Area::new_identical(
            VirAddr::from(srodata as usize).floor_to_vir_page_num(),
            VirAddr::from(erodata as usize).ceil_to_vir_page_num(),
            MappingPermission::R,
        ));

        // map .data section
        println!(
            "[kernel] Mapping .data section [{:#x}, {:#x})",
            sdata as usize, edata as usize
        );
        result.map(Area::new_identical(
            VirAddr::from(sdata as usize).floor_to_vir_page_num(),
            VirAddr::from(edata as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        // map .bss section
        println!(
            "[kernel] Mapping .bss section [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        result.map(Area::new_identical(
            VirAddr::from(sbss_with_stack as usize).floor_to_vir_page_num(),
            VirAddr::from(ebss as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        println!(
            "[kernel] Mapping allocated section [{:#x}, {:#x})",
            ekernel as usize, MEMORY_END,
        );
        result.map(Area::new_identical(
            VirAddr::from(ekernel as usize).floor_to_vir_page_num(),
            VirAddr::from(MEMORY_END).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        println!(
            "[kernel] Mapping memory-mapped registers for IO [{:#x}, {:#x})",
            UART_BASE_ADDRESS,
            UART_BASE_ADDRESS + UART_MAP_SIZE,
        );
        result.map(Area::new_identical(
            VirAddr::from(UART_BASE_ADDRESS).floor_to_vir_page_num(),
            VirAddr::from(UART_BASE_ADDRESS + UART_MAP_SIZE).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        println!(
            "[kernel] Mapping trampoline [{:#x}, {:#x})",
            usize::from(VirAddr::from(TRAMPOLINE_START_ADDRESS)),
            usize::from(VirAddr::from(TRAMPOLINE_START_ADDRESS) + PAGE_SIZE),
        );
        result.map_trampoline();

        result
    }

    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut result = Self::empty();

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
                println!(
                    "[mem] Start va is {:#x} and end va is {:#x}",
                    usize::from(start_va),
                    usize::from(end_va)
                );
                let area = Area::new_framed(
                    start_va.floor_to_vir_page_num(),
                    end_va.ceil_to_vir_page_num(),
                    map_perm,
                );
                end_vpn = end_va.ceil_to_vir_page_num();
                area.copy_from_raw_bytes(
                    &elf_file.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
                result.map(area);
            }
        }

        result.map_trampoline();
        result.map_trap_context();

        // map user stack
        let start_va: VirAddr = (end_vpn + 1).into(); // for guard page
        let end_va = start_va + USER_STACK_SIZE;
        result.map(Area::new_framed(
            start_va.floor_to_vir_page_num(),
            end_va.ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W | MappingPermission::U,
        ));

        (
            result,
            end_va.into(),
            elf_file.header.pt2.entry_point() as usize,
        )
    }
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        let mut result = Self::empty();
        self.areas.iter().for_each(|area| {
            let new_area = area.clone();
            if area.map_type() == MappingType::Framed {
                new_area.copy_from_existed(area);
            }
            result.map(new_area);
        });
        result.map_trampoline();
        result
    }
}

impl Memory {
    pub fn map(&mut self, area: Area) {
        println!(
            "[mem] Map area [{:#x}, {:#x})",
            area.range().start.0,
            area.range().end.0,
        );
        area.range().iter().enumerate().for_each(|(i, vpn)| {
            self.page_table
                .map(vpn, area.frame(i).ppn(), area.map_perm().into());
        });
        self.areas.push(area);
    }

    pub fn unmap(&mut self, vpn: VirPageNum) {
        let pos = self
            .areas
            .iter()
            .position(|area| area.range().start == vpn)
            .expect("[memory_set] Fail to find vpn in areas.");
        let area = &self.areas[pos];
        area.range().iter().for_each(|vpn| {
            self.page_table.unmap(vpn);
        });
        self.areas.remove(pos);
    }

    fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }
        println!(
            "trampoline: [{:#x}, {:#x})",
            TRAMPOLINE_START_ADDRESS,
            TRAMPOLINE_START_ADDRESS + PAGE_SIZE,
        );
        self.page_table.map(
            VirAddr::from(TRAMPOLINE_START_ADDRESS).floor_to_vir_page_num(),
            PhyAddr::from(strampoline as usize).floor_to_phy_page_num(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    fn map_trap_context(&mut self) {
        self.map(Area::new_framed(
            VirAddr::from(TRAP_CONTEXT_START_ADDRESS).floor_to_vir_page_num(),
            VirAddr::from(TRAP_CONTEXT_END_ADDRESS).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));
    }

    pub fn page_table(&self) -> &PageTable {
        &self.page_table
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: Mutex<Memory> = Mutex::new(Memory::new_kernel());
}
