use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Spin;

use core::arch::asm;
use core::cmp::{max, min};
use core::mem::size_of;

use super::address::{GenOffset, PhyAddr, VirAddr};
use super::area::Area;
use super::memory::MemSet;
use super::{
    address::{PhyPageNum, VirPageNum},
    frame::Frame,
    memory::MappingPermission,
};
use crate::config::{
    CLINT, MEMORY_END, PAGE_SIZE, PPN_WIDTH, PTE_FLAG_WIDTH, TRAMPOLINE_ADDR, UART_BASE_ADDRESS,
    UART_MAP_SIZE, VIRTIO_ADDR, VIRTIO_SIZE, VIRT_PLIC_ADDR, VIRT_PLIC_SIZE, VIRT_TEST,
    VIRT_TEST_SIZE,
};
use crate::fs::segment::Segment;
use crate::mm::memory::KERNEL_MEMSET;
use crate::proc::stack::UserStack;
use crate::trap::context::TrapCtxHandle;

bitflags! {
    /// This data structure is used to define the flags of page table entry.
    ///
    /// The smallest granularity of memory management is MappingArea, which is defined in `memory.rs`.
    /// Currently, it's meaningless to reduce the granularity to page table entry.
    pub struct PTEFlags: u8 {
        const V = 1 << 0; // Bit used to indicate validity.
        const R = 1 << 1; // Bit used to indicate readability.
        const W = 1 << 2; // Bit used to indicate writability.
        const X = 1 << 3; // Bit used to indicate whether executable.
        const U = 1 << 4; // Bit used to indicate user's accessibility.
        const G = 1 << 5; // Unknown.
        const A = 1 << 6; // Bit used to indicate whether page has been accessed since last reset.
        const D = 1 << 7; // Bit used to indicate whether page has been modified since last reset.
    }
}

/// Page table entry structure.
///
/// Only contain bits to cater memory layout required by SV39.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

/// The page table is only an abstraction towards the real page tables interleaved by page table entries.
/// It could be seen as an entry point for normal programs for stuffs that dealing with mapping.
///
/// RAII is used here. The frame collections control when to free those allocated frames used by page tables.
pub struct PageTable {
    root: PhyPageNum,
    frames: Spin<Vec<Frame>>,
}

impl PageTable {
    pub fn new_kernel(self: &Arc<Self>) -> MemSet {
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

        let mut areas = Vec::new();

        // map .text section
        areas.push(self.new_identical_area(
            VirAddr::from(stext as usize).floor_to_vir_page_num(),
            VirAddr::from(etext as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::X,
        ));

        // map .rodata section
        areas.push(self.new_identical_area(
            VirAddr::from(srodata as usize).floor_to_vir_page_num(),
            VirAddr::from(erodata as usize).ceil_to_vir_page_num(),
            MappingPermission::R,
        ));

        // map .data section
        areas.push(self.new_identical_area(
            VirAddr::from(sdata as usize).floor_to_vir_page_num(),
            VirAddr::from(edata as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        // map .bss section
        areas.push(self.new_identical_area(
            VirAddr::from(sbss_with_stack as usize).floor_to_vir_page_num(),
            VirAddr::from(ebss as usize).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        areas.push(self.new_identical_area(
            VirAddr::from(ekernel as usize).floor_to_vir_page_num(),
            VirAddr::from(MEMORY_END).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        areas.push(self.new_identical_area(
            VirAddr::from(UART_BASE_ADDRESS).floor_to_vir_page_num(),
            VirAddr::from(UART_BASE_ADDRESS + UART_MAP_SIZE).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        areas.push(self.new_identical_area(
            VirAddr::from(VIRTIO_ADDR).floor_to_vir_page_num(),
            VirAddr::from(VIRTIO_ADDR + VIRTIO_SIZE).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        areas.push(self.new_identical_area(
            VirAddr::from(CLINT).floor_to_vir_page_num(),
            VirAddr::from(CLINT + 0x10000).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        areas.push(self.new_identical_area(
            VirAddr::from(VIRT_PLIC_ADDR).floor_to_vir_page_num(),
            VirAddr::from(VIRT_PLIC_ADDR + VIRT_PLIC_SIZE).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        areas.push(self.new_identical_area(
            VirAddr::from(VIRT_TEST).floor_to_vir_page_num(),
            VirAddr::from(VIRT_TEST + VIRT_TEST_SIZE).ceil_to_vir_page_num(),
            MappingPermission::R | MappingPermission::W,
        ));

        self.map_trampoline();

        MemSet::new(areas)
    }

    pub fn new_user(self: &Arc<Self>, elf_data: &[u8]) -> (VirAddr, VirAddr, MemSet) {
        let mut areas = Vec::new();

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
                let area = self.new_framed_area(
                    start_va.floor_to_vir_page_num(),
                    end_va.ceil_to_vir_page_num(),
                    map_perm,
                );
                end_vpn = end_va.ceil_to_vir_page_num();
                area.copy_from_raw_bytes(
                    &elf_file.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
                areas.push(area);
            }
        }

        self.map_trampoline();

        let base: VirAddr = (end_vpn + 1).into(); // for guard page
        (
            base,
            (elf_file.header.pt2.entry_point() as usize).into(),
            MemSet::new(areas),
        )
    }
}

impl PageTable {
    pub fn new_identical_area(
        self: &Arc<Self>,
        start_vpn: VirPageNum,
        end_vpn: VirPageNum,
        map_perm: MappingPermission,
    ) -> Area {
        Area::new_identical(start_vpn, end_vpn, map_perm, self)
    }

    pub fn new_framed_area(
        self: &Arc<Self>,
        start_vpn: VirPageNum,
        end_vpn: VirPageNum,
        map_perm: MappingPermission,
    ) -> Area {
        Area::new_framed(start_vpn, end_vpn, map_perm, self)
    }

    pub fn new_linear_area(
        self: &Arc<Self>,
        start_vpn: VirPageNum,
        start_ppn: PhyPageNum,
        len: usize,
        map_perm: MappingPermission,
    ) -> Area {
        Area::new_linear(start_vpn, start_ppn, len, map_perm, self)
    }

    pub fn new_trap_ctx(self: &Arc<Self>, tid: usize) -> TrapCtxHandle {
        TrapCtxHandle::new(tid, self)
    }

    pub fn new_user_stack(self: &Arc<Self>, base: VirAddr, tid: usize) -> UserStack {
        UserStack::new(base, tid, self)
    }

    pub fn map_trampoline(self: &Self) {
        extern "C" {
            fn strampoline();
        }
        self.map(
            VirAddr::from(TRAMPOLINE_ADDR).floor_to_vir_page_num(),
            PhyAddr::from(strampoline as usize).floor_to_phy_page_num(),
            PTEFlags::R | PTEFlags::X,
        )
    }
}

impl PageTable {
    pub fn new() -> Self {
        let frame = Frame::new();
        println!(
            "[page_table] Root of page table is {:#x}.",
            usize::from(frame.ppn())
        );
        Self {
            root: frame.ppn(),
            frames: Spin::new(vec![frame]),
        }
    }

    pub fn map(&self, vpn: VirPageNum, ppn: PhyPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn);
        pte.set_ppn(ppn);
        pte.set_flags(flags | PTEFlags::V);
    }

    pub fn map_area(&self, area: &Area) {
        println!(
            "[mem] Map area [{:#x}, {:#x})",
            usize::from(area.range().start),
            usize::from(area.range().end),
        );
        let flags = area.map_perm().into();
        area.range()
            .iter()
            .zip(area.frames().iter())
            .for_each(|(vpn, frame)| {
                self.map(vpn, frame.ppn(), flags);
            });
    }

    pub fn unmap(&self, vpn: VirPageNum) {
        // TODO: some frames in page table might never be used again, hence deallocation is meaningful
        let pte = self
            .find_pte(vpn)
            .expect("[page_table] Unmap a non-exist page table entry.");
        pte.set_flags(PTEFlags::empty());
    }

    pub fn unmap_area(&self, area: &Area) {
        println!(
            "[mem] Unmap area [{:#x}, {:#x})",
            usize::from(area.range().start),
            usize::from(area.range().end),
        );
        area.range().iter().for_each(|vpn| self.unmap(vpn));
    }

    /// Convert the root physical page number of page table to a form that can be used by `satp` register.
    pub fn to_satp(&self) -> usize {
        8usize << 60 | self.root.0
    }

    /// Find the page table entry with given virtual page number, creating new page table entry when necessary.
    fn create_pte(&self, vpn: VirPageNum) -> &mut PageTableEntry {
        let indices = vpn.indices();
        let mut ptes = self.root.as_raw_ptes();
        for (i, idx) in indices.iter().enumerate() {
            let pte = &mut ptes[*idx];
            if i == 2 {
                return pte;
            }
            if !pte.is_valid() {
                let frame = Frame::new();
                pte.set_ppn(frame.ppn());
                pte.set_flags(PTEFlags::V);
                self.frames.lock().push(frame);
            }
            ptes = pte.get_ppn().as_raw_ptes();
        }
        unreachable!();
    }

    /// Find the page table entry with given virtual page number without creating new page table entry on the way.
    fn find_pte(&self, vpn: VirPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.indices();
        let mut ptes = self.root.as_raw_ptes();
        for (i, idx) in indices.iter().enumerate() {
            let pte = &mut ptes[*idx];
            if !pte.is_valid() {
                return None;
            }
            if i == 2 {
                return Some(pte);
            }
            ptes = pte.get_ppn().as_raw_ptes();
        }
        unreachable!();
    }
}

impl PageTable {
    pub fn translate_va(&self, va: VirAddr) -> Option<PhyAddr> {
        let vpn = VirPageNum::from(va);
        let offset = GenOffset::from(va);
        let ppn = self.find_pte(vpn)?.get_ppn();
        Some(ppn + offset)
    }

    pub fn translate_vpn(&self, vpn: VirPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub fn translate_bytes(&self, ptr: VirAddr, len: usize) -> Vec<&'static mut u8> {
        let ptr = usize::from(ptr);
        let mut vpn = VirPageNum::from(ptr);
        let mut result: Vec<&'static mut u8> = Vec::new();
        while usize::from(vpn) < ptr as usize + len {
            let ppn = self.find_pte(vpn).unwrap().get_ppn();
            let start = max(ptr - usize::from(vpn), 0);
            let end = min(ptr + len - usize::from(vpn), PAGE_SIZE);
            ppn.as_raw_bytes()[start..end]
                .iter_mut()
                .for_each(|byte| result.push(byte));
            vpn += 1;
        }
        result
    }

    pub fn translate_segment(&self, ptr: VirAddr, len: usize) -> Segment {
        let ptr = usize::from(ptr);
        let mut vpn = VirPageNum::from(ptr);
        let mut result: Vec<&'static mut [u8]> = Vec::new();
        while usize::from(vpn) < ptr as usize + len {
            let ppn = self.find_pte(vpn).unwrap().get_ppn();
            let start = max(ptr - usize::from(vpn), 0);
            let end = min(ptr + len - usize::from(vpn), PAGE_SIZE);
            result.push(&mut ppn.as_raw_bytes()[start..end]);
            vpn += 1;
        }
        Segment::new(result)
    }

    pub fn translate_str(&self, ptr: VirAddr) -> String {
        let mut ptr = usize::from(ptr);
        let mut vpn = VirPageNum::from(ptr);
        ptr -= usize::from(vpn);
        let mut result = String::new();
        loop {
            let ppn = self.find_pte(vpn).unwrap().get_ppn();
            let bytes = ppn.as_raw_bytes();
            let mut c = bytes[ptr];
            loop {
                if c == '\0' as u8 {
                    return result;
                }
                if ptr == PAGE_SIZE {
                    break;
                }
                result.push(c as char);
                ptr += 1;
                c = bytes[ptr];
            }
            vpn += 1;
            ptr = 0;
        }
    }

    pub fn translate_ptr(&self, ptr: VirAddr) -> &'static mut u8 {
        let vpn = VirPageNum::from(ptr);
        let ppn = self.find_pte(vpn).unwrap().get_ppn();
        let offset = usize::from(ptr) - usize::from(vpn);
        &mut ppn.as_raw_bytes()[offset]
    }

    /// This function translate a piece of virtual memory into the type specified within just one page.
    ///
    /// It doesn't support translation across pages.  
    pub fn translate_any<T>(&self, ptr: VirAddr) -> &'static mut T {
        let size = size_of::<T>();
        let vpn = VirPageNum::from(ptr);
        let ppn = self.find_pte(vpn).unwrap().get_ppn();
        let offset = usize::from(ptr) - usize::from(vpn);
        assert!(offset + size <= PAGE_SIZE);
        unsafe {
            (&mut ppn.as_raw_bytes()[offset] as *mut u8 as *mut T)
                .as_mut()
                .unwrap()
        }
    }

    pub fn translate_array<T>(&self, ptr: VirAddr, len: usize) -> Vec<&'static mut T> {
        let mut res = Vec::new();
        for i in 0..len {
            let ptr = ptr + i * size_of::<T>();
            res.push(self.translate_any(ptr));
        }
        res
    }
}

impl Drop for PageTable {
    fn drop(&mut self) {
        self.frames.lock().iter().for_each(|frame| drop(frame));
    }
}

impl From<MappingPermission> for PTEFlags {
    fn from(value: MappingPermission) -> Self {
        PTEFlags::from_bits(value.bits()).unwrap()
    }
}

pub fn activate_page_table() {
    assert_ne!(KERNEL_MEMSET.len(), 0);
    let satp = KERNEL_PAGE_TABLE.to_satp();
    riscv::register::satp::write(satp);
    unsafe {
        asm!("sfence.vma");
    }
}

impl PageTableEntry {
    pub fn new(ppn: PhyPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << PTE_FLAG_WIDTH | flags.bits as usize,
        }
    }

    pub fn get_ppn(&self) -> PhyPageNum {
        let mut bits = self.bits;
        bits = bits >> PTE_FLAG_WIDTH;
        bits = bits & ((1 << PPN_WIDTH) - 1);
        PhyPageNum(bits)
    }

    pub fn set_ppn(&mut self, ppn: PhyPageNum) {
        let flags = self.bits & ((1 << PTE_FLAG_WIDTH) - 1);
        self.bits = ppn.0 << PTE_FLAG_WIDTH | flags;
    }

    pub fn set_flags(&mut self, flags: PTEFlags) {
        let ppn = self.get_ppn();
        self.bits = ppn.0 << PTE_FLAG_WIDTH | flags.bits as usize;
    }

    pub fn get_flags(&self) -> PTEFlags {
        // truncate
        PTEFlags::from_bits(self.bits as u8)
            .expect("[page_table] Try to convert an invalid page table entry.")
    }

    pub fn is_valid(&self) -> bool {
        self.get_flags().contains(PTEFlags::V)
    }
}

lazy_static! {
    pub static ref KERNEL_PAGE_TABLE: Arc<PageTable> = Arc::new(PageTable::new());
}
