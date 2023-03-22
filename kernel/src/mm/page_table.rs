use alloc::vec::Vec;
use bitflags::bitflags;

use core::arch::asm;
use core::cmp::{max, min};

use super::memory::KERNEL_SPACE;
use super::{
    address::{PhyPageNum, VirPageNum},
    frame::Frame,
    memory::MappingPermission,
};
use crate::config::{PAGE_SIZE, PPN_WIDTH, PTE_FLAG_WIDTH};
use crate::println;

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

/// The page table is only an abstraction towards the real page tables interleaved by page table entries.
/// It could be seen as an entry point for normal programs for stuffs that dealing with mapping.
///
/// RAII is used here. The frame collections control when to free those allocated frames used by page tables.
pub struct PageTable {
    root: PhyPageNum,
    frames: Vec<Frame>,
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
            frames: vec![frame],
        }
    }

    pub fn map(&mut self, vpn: VirPageNum, ppn: PhyPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn);
        pte.set_ppn(ppn);
        pte.set_flags(flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirPageNum) {
        // TODO: some frames in page table might never be used again, hence deallocation is meaningful
        let pte = self
            .find_pte(vpn)
            .expect("[page_table] Unmap a non-exist page table entry.");
        pte.set_flags(PTEFlags::empty());
    }

    pub fn translate(&self, vpn: VirPageNum) -> Option<PageTableEntry> {
        let indices = vpn.indices();
        self.find_pte(vpn).map(|pte| *pte)
    }

    /// Convert the root physical page number of page table to a form that can be used by `satp` register.
    pub fn to_satp(&self) -> usize {
        8usize << 60 | self.root.0
    }

    /// Find the page table entry with given virtual page number, creating new page table entry when necessary.
    fn create_pte(&mut self, vpn: VirPageNum) -> &mut PageTableEntry {
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
                self.frames.push(frame);
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

impl Drop for PageTable {
    fn drop(&mut self) {
        self.frames.iter().for_each(|frame| drop(frame));
    }
}

impl From<MappingPermission> for PTEFlags {
    fn from(value: MappingPermission) -> Self {
        PTEFlags::from_bits(value.bits()).unwrap()
    }
}

pub fn activate_page_table() {
    let satp = KERNEL_SPACE.lock().page_table().to_satp();
    riscv::register::satp::write(satp);
    unsafe {
        asm!("sfence.vma");
    }
}

pub fn translate_bytes(page_table: &PageTable, ptr: *const u8, len: usize) -> Vec<u8> {
    let ptr = ptr as usize;
    let mut vpn = VirPageNum::from(ptr);
    let mut result: Vec<u8> = Vec::new();
    while usize::from(vpn) <= ptr as usize + len {
        let ppn = page_table.find_pte(vpn).unwrap().get_ppn();
        let start = max(ptr - usize::from(vpn), 0);
        let end = min(ptr + len - usize::from(vpn), PAGE_SIZE);
        ppn.as_raw_bytes()[start..end]
            .iter()
            .for_each(|byte| result.push(*byte));
        vpn += 1;
    }
    result
}
