use core::sync::atomic::AtomicUsize;

use super::Page;

/// One possible representation of metadata of page, when the page is used by the page table.
///
/// Here, **pt** stands for page table.
#[derive(Debug)]
pub struct PtPage {
    pa: AtomicUsize,
}

pub struct PtPagePtr {
    pa: usize,
}
