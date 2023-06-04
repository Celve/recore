use core::ops::{Deref, DerefMut};

use allocator::linked_list::LinkedList;
use spin::SpinGuard;

use crate::{
    config::PAGE_SIZE,
    mem::{Page, Pageable},
    mm::address::PhyPageNum,
};

/// One possible representation of metadata of page, when the page is used by slab allocator.
///
/// We need to guarantee that all operations toward it is atomic.
#[derive(Debug)]
pub struct SlabPage {
    pa: usize,

    order: u8,

    pub prev: PhyPageNum,
    pub next: PhyPageNum,
    pub free: LinkedList,
    pub inuse: u8,
}

/// Automatically fetch the lock, returning the guard.
pub struct SlabPageGuard<'a> {
    pub ppn: PhyPageNum,
    guard: SpinGuard<'a, Page>,
}

impl SlabPage {
    pub fn new(pa: usize, order: u8) -> SlabPage {
        let mut slab_page = Self {
            pa,
            order,
            prev: PhyPageNum::null(),
            next: PhyPageNum::null(),
            free: LinkedList::new(),
            inuse: 0,
        };
        slab_page.init();
        slab_page
    }

    pub fn alloc(pa: usize, order: u8) {
        let mut slab_page = Self {
            pa,
            order,
            prev: PhyPageNum::null(),
            next: PhyPageNum::null(),
            free: LinkedList::new(),
            inuse: 0,
        };
        slab_page.init();
        *Page::from_pa(pa).lock() = Page::Slab(slab_page);
    }

    pub fn init(&mut self) {
        let size = 1 << self.order;
        self.free = LinkedList::new();
        (0..PAGE_SIZE / size).rev().for_each(|i| {
            unsafe { self.free.push_front((self.pa + i * size) as *mut usize) };
        });
    }

    /// Take a free object inside slab with `inused` increased.
    pub fn take_slot(&mut self) -> Option<*mut usize> {
        let res = self.free.pop_front();
        if res.is_some() {
            self.inuse += 1;
        }
        res
    }

    /// Insert a free object inside slab with `inused` decreased.
    ///
    /// Make sure that the pointer is valid and unique.
    pub unsafe fn return_slot(&mut self, ptr: *mut usize) {
        self.inuse -= 1;
        unsafe {
            self.free.push_front(ptr);
        }
    }

    pub fn is_available(&mut self) -> bool {
        !self.free.is_empty()
    }
}

impl Pageable for SlabPage {
    fn new_page(pa: PhyPageNum) -> Page {
        Page::Slab(SlabPage::new(pa.into(), 0))
    }
}

impl<'a> SlabPageGuard<'a> {
    pub fn new(ppn: PhyPageNum) -> Self {
        let guard = Page::from_ppn(ppn).lock();
        Self { ppn, guard }
    }
}

impl<'a> Deref for SlabPageGuard<'a> {
    type Target = SlabPage;

    fn deref(&self) -> &Self::Target {
        self.guard.as_slab()
    }
}

impl<'a> DerefMut for SlabPageGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_slab_mut()
    }
}
