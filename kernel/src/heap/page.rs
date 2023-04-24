use core::ops::{Deref, DerefMut};

use allocator::linked_list::LinkedList;

use crate::config::PAGE_SIZE;

#[derive(Clone, Copy)]
pub struct Page {
    // metadata for page
    pa: usize,

    // metadata for slub
    order: usize,
    prev: Option<PagePtr>,
    next: Option<PagePtr>,
    pub free: LinkedList,
    inuse: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PagePtr {
    ptr: *mut Page,
}

impl Page {
    pub fn make_slab(&mut self) {
        let size = 1 << self.order;
        (0..PAGE_SIZE / size).rev().for_each(|i| {
            unsafe { self.free.push_front((self.pa + i * size) as *mut usize) };
        });
    }
}

impl Page {
    pub const fn empty() -> Self {
        Self {
            pa: 0,
            order: 0,
            prev: None,
            next: None,
            free: LinkedList::new(),
            inuse: 0,
        }
    }

    pub fn pa(&mut self) -> usize {
        self.pa
    }

    pub fn pa_mut(&mut self) -> &mut usize {
        &mut self.pa
    }

    pub fn prev(&self) -> Option<PagePtr> {
        self.prev
    }

    pub fn prev_insert(&mut self, prev: Option<PagePtr>) {
        self.prev = prev;
    }

    pub fn next(&self) -> Option<PagePtr> {
        self.next
    }

    pub fn next_insert(&mut self, next: Option<PagePtr>) {
        self.next = next;
    }

    /// Take a free object inside slab with `inused` increased.
    pub fn take_free(&mut self) -> Option<*mut usize> {
        let res = self.free.pop_front();
        if res.is_some() {
            self.inuse += 1;
        }
        res
    }

    /// Insert a free object inside slab with `inused` decreased.
    pub fn insert_free(&mut self, ptr: *mut usize) {
        self.inuse -= 1;
        unsafe {
            self.free.push_front(ptr);
        }
    }

    pub fn is_free(&self) -> bool {
        !self.free.is_empty()
    }

    pub fn order_mut(&mut self) -> &mut usize {
        &mut self.order
    }

    pub fn inuse(&self) -> usize {
        self.inuse
    }
}

impl PagePtr {
    pub fn new(ptr: *mut Page) -> Self {
        Self { ptr }
    }
}

impl Deref for PagePtr {
    type Target = Page;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for PagePtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

/// PagePtr is promised by slab allocator that it would only be accessed by one thread at one time.
/// Let alone I don't implement multi-thread.
unsafe impl Sync for PagePtr {}
unsafe impl Send for PagePtr {}
