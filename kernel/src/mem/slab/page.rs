use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use allocator::linked_list::LinkedList;

use crate::{
    config::PAGE_SIZE,
    mem::{section::MemSec, Page, Pageable},
    mm::address::PhyPageNum,
};

/// One possible representation of metadata of page, when the page is used by slab allocator.
///
/// We need to guarantee that all operations toward it is atomic.
#[derive(Debug)]
pub struct SlabPage {
    pa: usize,

    order: u8,

    pub prev: SlabPagePtr,
    pub next: SlabPagePtr,
    pub free: LinkedList,
    pub inuse: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SlabPagePtr {
    pa: usize,
}

/// It's guaranteed that the slab page would only be accessed by one thread at a time.
unsafe impl Sync for SlabPage {}

impl SlabPage {
    pub fn new(pa: usize, order: u8) -> SlabPage {
        let mut slab_page = Self {
            pa,
            order,
            prev: SlabPagePtr::null(),
            next: SlabPagePtr::null(),
            free: LinkedList::new(),
            inuse: 0,
        };
        slab_page.init();
        slab_page
    }

    pub fn alloc(pa: usize, order: u8) -> SlabPagePtr {
        let mut slab_page = Self {
            pa,
            order,
            prev: SlabPagePtr::null(),
            next: SlabPagePtr::null(),
            free: LinkedList::new(),
            inuse: 0,
        };
        slab_page.init();
        *Page::from_addr_mut(pa) = Page::Slab(slab_page);

        SlabPagePtr { pa }
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

impl SlabPagePtr {
    pub fn new(pa: usize) -> Self {
        Self { pa }
    }
}

impl SlabPagePtr {
    pub const fn null() -> Self {
        Self { pa: 0 }
    }

    pub fn pa(&self) -> usize {
        self.pa
    }

    pub fn is_null(&self) -> bool {
        self.pa == 0
    }
}

impl Deref for SlabPagePtr {
    type Target = SlabPage;

    fn deref(&self) -> &Self::Target {
        Page::from_addr(self.pa).as_slab()
    }
}

impl DerefMut for SlabPagePtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Page::from_addr_mut(self.pa).as_slab_mut()
    }
}
