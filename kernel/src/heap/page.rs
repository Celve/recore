use core::ops::{Deref, DerefMut};

use allocator::linked_list::LinkedList;

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

use super::{KERNEL_HEAP_SPACE, MEM_MAP};

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

/// PagePtr is necessary because it would be used across threads.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PagePtr {
    ptr: *mut Page,
}

impl Page {
    pub fn make_slab(&mut self) {
        let size = 1 << self.order;
        self.free = LinkedList::new();
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
    ///
    /// Make sure that the pointer is valid and unique.
    pub unsafe fn insert_free(&mut self, ptr: *mut usize) {
        self.inuse -= 1;
        unsafe {
            self.free.push_front(ptr);
        }
    }

    /// Check whether the linked list is empty.
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
    /// Return a page pointer that points to the page governing the given address.
    pub fn new(ptr: usize) -> Self {
        unsafe {
            let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
            Self {
                ptr: &mut MEM_MAP[(ptr - start) >> PAGE_SIZE_BITS],
            }
        }
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
