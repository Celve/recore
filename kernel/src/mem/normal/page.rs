use core::ops::{Deref, DerefMut};

use spin::SpinGuard;

use crate::{
    config::PAGE_SIZE,
    mem::{allocator::PageAllocator, page::Page},
    mm::address::PhyPageNum,
};

use super::FRAME_ALLOCATOR;

/// One possible representation of metadata of page, when the page is used by the page table.
///
/// Here, **pt** stands for page table.
#[derive(Debug)]
pub struct NormalPage {
    pub ppn: PhyPageNum,
    pub refcnt: usize,
}

pub struct NormalPageHandle {
    pub ppn: PhyPageNum,
}

/// Automatically fetch the lock, returning the guard.
pub struct NormalPageGuard<'a> {
    pub ppn: PhyPageNum,
    guard: SpinGuard<'a, Page>,
}

impl NormalPage {
    /// Create a page with referencing count is equal to 1.
    pub fn alloc(ppn: PhyPageNum) {
        *Page::from_ppn(ppn).lock() = Page::Normal(Self { ppn, refcnt: 1 });
    }
}

impl NormalPageHandle {
    pub fn new() -> Self {
        let ppn = FRAME_ALLOCATOR.alloc_page();
        NormalPage::alloc(ppn);

        // init
        let handle = Self { ppn };
        handle.init();
        handle
    }

    pub fn init(&self) {
        let ptr = usize::from(self.ppn) as *mut u8;
        unsafe {
            core::slice::from_raw_parts_mut(ptr, PAGE_SIZE).fill(0);
        }
    }
}

impl Drop for NormalPageHandle {
    fn drop(&mut self) {
        let cnt = {
            let mut page = NormalPageGuard::new(self.ppn);
            page.refcnt -= 1;
            page.refcnt
        };
        if cnt == 0 {
            FRAME_ALLOCATOR.dealloc_page(self.ppn);
        }
    }
}

impl<'a> NormalPageGuard<'a> {
    pub fn new(ppn: PhyPageNum) -> Self {
        let guard = Page::from_ppn(ppn).lock();
        Self { ppn, guard }
    }
}

impl<'a> Deref for NormalPageGuard<'a> {
    type Target = NormalPage;

    fn deref(&self) -> &Self::Target {
        self.guard.as_normal()
    }
}

impl<'a> DerefMut for NormalPageGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_normal_mut()
    }
}
