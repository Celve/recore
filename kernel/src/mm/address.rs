use super::page_table::PageTableEntry;
use super::range::Step;
use crate::config::*;
use core::{
    mem::size_of,
    ops::{self, AddAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhyAddr(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirAddr(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhyPageNum(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirPageNum(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenOffset(pub usize);

macro_rules! truncate_phy_addr {
    ($e: expr) => {
        ($e & ((1 << PA_WIDTH) - 1))
    };
}

macro_rules! truncate_vir_addr {
    ($e: expr) => {
        ($e & ((1 << VA_WIDTH) - 1))
    };
}

macro_rules! truncate_page_num {
    ($e: expr) => {
        ($e & !((1 << PAGE_SIZE_BITS) - 1))
    };
}

macro_rules! truncate_offset {
    ($e: expr) => {
        ($e & ((1 << PAGE_SIZE_BITS) - 1))
    };
}

// from usize to any

impl From<usize> for PhyAddr {
    fn from(value: usize) -> Self {
        Self(truncate_phy_addr!(value))
    }
}

impl From<usize> for VirAddr {
    fn from(value: usize) -> Self {
        Self(truncate_vir_addr!(value))
    }
}

impl From<usize> for PhyPageNum {
    fn from(value: usize) -> Self {
        Self(truncate_page_num!(truncate_phy_addr!(value)) >> PAGE_SIZE_BITS)
    }
}

impl From<usize> for VirPageNum {
    fn from(value: usize) -> Self {
        Self(truncate_page_num!(truncate_vir_addr!(value)) >> PAGE_SIZE_BITS)
    }
}

impl From<usize> for GenOffset {
    fn from(value: usize) -> Self {
        Self(truncate_offset!(value))
    }
}

// from any to usize

impl From<PhyAddr> for usize {
    fn from(value: PhyAddr) -> Self {
        value.0
    }
}

impl From<VirAddr> for usize {
    fn from(value: VirAddr) -> Self {
        if value.0 >> (VA_WIDTH - 1) != 0 {
            value.0 | !((1 << VA_WIDTH) - 1)
        } else {
            value.0
        }
    }
}

impl From<PhyPageNum> for usize {
    fn from(value: PhyPageNum) -> Self {
        value.0 << PAGE_SIZE_BITS
    }
}

impl From<VirPageNum> for usize {
    fn from(value: VirPageNum) -> Self {
        let va = value.0 << PAGE_SIZE_BITS;
        if va >> (VA_WIDTH - 1) != 0 {
            va | !((1 << VA_WIDTH) - 1)
        } else {
            va
        }
    }
}

impl From<GenOffset> for usize {
    fn from(value: GenOffset) -> Self {
        value.0
    }
}

// from any to any

impl From<PhyPageNum> for PhyAddr {
    fn from(value: PhyPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl From<VirAddr> for PhyAddr {
    fn from(value: VirAddr) -> Self {
        Self(value.0)
    }
}

impl From<VirPageNum> for VirAddr {
    fn from(value: VirPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl From<PhyAddr> for VirAddr {
    fn from(value: PhyAddr) -> Self {
        Self(value.0)
    }
}

impl From<PhyAddr> for PhyPageNum {
    fn from(value: PhyAddr) -> Self {
        Self(truncate_page_num!(value.0) >> PAGE_SIZE_BITS)
    }
}

impl From<VirPageNum> for PhyPageNum {
    fn from(value: VirPageNum) -> Self {
        Self(value.0)
    }
}

impl From<VirAddr> for VirPageNum {
    fn from(value: VirAddr) -> Self {
        Self(truncate_page_num!(value.0) >> PAGE_SIZE_BITS)
    }
}

impl From<PhyPageNum> for VirPageNum {
    fn from(value: PhyPageNum) -> Self {
        Self(value.0)
    }
}

impl From<PhyAddr> for GenOffset {
    fn from(value: PhyAddr) -> Self {
        Self(truncate_offset!(value.0))
    }
}

impl From<VirAddr> for GenOffset {
    fn from(value: VirAddr) -> Self {
        Self(truncate_offset!(value.0))
    }
}

// utilities

impl PhyAddr {
    pub fn new(ppn: PhyPageNum, go: GenOffset) -> Self {
        Self(ppn.0 << PAGE_SIZE_BITS | go.0)
    }

    pub fn phy_page_num(&self) -> PhyPageNum {
        PhyPageNum(truncate_page_num!(self.0) >> PAGE_SIZE_BITS)
    }

    pub fn gen_offset(&self) -> GenOffset {
        GenOffset(truncate_offset!(self.0))
    }

    pub fn ceil_to_phy_page_num(&self) -> PhyPageNum {
        (self.0 + PAGE_SIZE - 1).into()
    }

    /// This function is equivalent to `phy_page_num()`, with different semantic.
    pub fn floor_to_phy_page_num(&self) -> PhyPageNum {
        PhyPageNum(truncate_page_num!(self.0) >> PAGE_SIZE_BITS)
    }

    pub fn as_ref<T>(&self) -> Option<&T> {
        unsafe { (self.0 as *mut T).as_ref() }
    }

    pub fn as_mut<T>(&self) -> Option<&mut T> {
        unsafe { (self.0 as *mut T).as_mut() }
    }
}

impl VirAddr {
    pub fn new(vpn: VirPageNum, go: GenOffset) -> Self {
        Self(vpn.0 << PAGE_SIZE_BITS | go.0)
    }

    pub fn vir_page_num(&self) -> VirPageNum {
        VirPageNum(truncate_page_num!(self.0) >> PAGE_SIZE_BITS)
    }

    pub fn gen_offset(&self) -> GenOffset {
        GenOffset(truncate_offset!(self.0))
    }

    pub fn ceil_to_vir_page_num(&self) -> VirPageNum {
        (self.0 + PAGE_SIZE - 1).into()
    }

    pub fn floor_to_vir_page_num(&self) -> VirPageNum {
        VirPageNum(truncate_page_num!(self.0) >> PAGE_SIZE_BITS)
    }
}

impl PhyPageNum {
    pub fn as_raw_ptes(&self) -> &'static mut [PageTableEntry] {
        let start_ptr = usize::from(*self) as *mut PageTableEntry;
        unsafe {
            core::slice::from_raw_parts_mut(start_ptr, PAGE_SIZE / size_of::<PageTableEntry>())
        }
    }

    pub fn as_raw_bytes(&self) -> &'static mut [u8] {
        let start_ptr = usize::from(*self) as *mut u8;
        unsafe { core::slice::from_raw_parts_mut(start_ptr, PAGE_SIZE) }
    }
}

impl VirPageNum {
    /// Convert virtual page number to three parts for page table to index.
    ///
    /// Please pay attention to the order, which I have made a mistake.
    pub fn indices(&self) -> [usize; 3] {
        let mask = (1 << 9) - 1;
        let l2 = self.0 & mask;
        let l1 = self.0 >> 9 & mask;
        let l0 = self.0 >> 18 & mask;
        [l0, l1, l2]
    }
}

// operator overloading

impl ops::Add<usize> for PhyAddr {
    type Output = PhyAddr;

    fn add(self, rhs: usize) -> Self::Output {
        PhyAddr(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for PhyAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl ops::Sub<usize> for PhyAddr {
    type Output = PhyAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        PhyAddr(self.0 - rhs)
    }
}

impl ops::Sub<PhyAddr> for PhyAddr {
    type Output = usize;

    fn sub(self, rhs: PhyAddr) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::SubAssign<usize> for PhyAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl ops::Add<usize> for VirAddr {
    type Output = VirAddr;

    fn add(self, rhs: usize) -> Self::Output {
        VirAddr(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for VirAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl ops::Sub<usize> for VirAddr {
    type Output = VirAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        VirAddr(self.0 - rhs)
    }
}

impl ops::Sub<VirAddr> for VirAddr {
    type Output = usize;

    fn sub(self, rhs: VirAddr) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::SubAssign<usize> for VirAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl ops::Add<usize> for PhyPageNum {
    type Output = PhyPageNum;

    fn add(self, rhs: usize) -> Self::Output {
        PhyPageNum(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for PhyPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl ops::Sub<usize> for PhyPageNum {
    type Output = PhyPageNum;

    fn sub(self, rhs: usize) -> Self::Output {
        PhyPageNum(self.0 - rhs)
    }
}

impl ops::Sub<PhyPageNum> for PhyPageNum {
    type Output = usize;

    fn sub(self, rhs: PhyPageNum) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::SubAssign<usize> for PhyPageNum {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl ops::Add<usize> for VirPageNum {
    type Output = VirPageNum;

    fn add(self, rhs: usize) -> Self::Output {
        VirPageNum(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for VirPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl ops::Sub<usize> for VirPageNum {
    type Output = VirPageNum;

    fn sub(self, rhs: usize) -> Self::Output {
        VirPageNum(self.0 - rhs)
    }
}

impl ops::Sub<VirPageNum> for VirPageNum {
    type Output = usize;

    fn sub(self, rhs: VirPageNum) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::SubAssign<usize> for VirPageNum {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl ops::Add<usize> for GenOffset {
    type Output = GenOffset;

    fn add(self, rhs: usize) -> Self::Output {
        GenOffset(self.0 + rhs)
    }
}

impl ops::AddAssign<usize> for GenOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl ops::Sub<usize> for GenOffset {
    type Output = GenOffset;

    fn sub(self, rhs: usize) -> Self::Output {
        GenOffset(self.0 - rhs)
    }
}

impl ops::Sub<GenOffset> for GenOffset {
    type Output = usize;

    fn sub(self, rhs: GenOffset) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::SubAssign<usize> for GenOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Step for PhyAddr {
    fn step(&mut self) {
        self.add_assign(1);
    }
}

impl Step for VirAddr {
    fn step(&mut self) {
        self.add_assign(1);
    }
}

impl Step for PhyPageNum {
    fn step(&mut self) {
        self.add_assign(1);
    }
}

impl Step for VirPageNum {
    fn step(&mut self) {
        self.add_assign(1);
    }
}

impl Step for GenOffset {
    fn step(&mut self) {
        self.add_assign(1);
    }
}
