use crate::config::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhyAddr(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirAddr(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhyPageNum(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirPageNum(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenOffset(usize);

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
        Self(truncate_page_num!(truncate_phy_addr!(value)))
    }
}

impl From<usize> for VirPageNum {
    fn from(value: usize) -> Self {
        Self(truncate_page_num!(truncate_vir_addr!(value)))
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
        value.0
    }
}

impl From<PhyPageNum> for usize {
    fn from(value: PhyPageNum) -> Self {
        value.0
    }
}

impl From<VirPageNum> for usize {
    fn from(value: VirPageNum) -> Self {
        value.0
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
        Self(value.0)
    }
}

impl From<VirPageNum> for VirAddr {
    fn from(value: VirPageNum) -> Self {
        Self(value.0)
    }
}

impl From<PhyAddr> for PhyPageNum {
    fn from(value: PhyAddr) -> Self {
        Self(truncate_page_num!(value.0))
    }
}

impl From<VirAddr> for VirPageNum {
    fn from(value: VirAddr) -> Self {
        Self(truncate_page_num!(value.0))
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
