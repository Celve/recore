use core::alloc::{GlobalAlloc, Layout};

use allocator::linked_list::LinkedList;

use super::{fetch_page, page::PagePtr, HEAP};

#[derive(Clone, Copy)]
pub struct Cache {
    order: usize,
    curr: Option<PagePtr>,
    next: Option<PagePtr>,
}

impl Cache {
    pub fn alloc(&mut self) -> usize {
        // if the curr is empty
        if self.curr.is_none() {
            self.curr = if let Some(mut page) = self.next {
                // find from existed, adjust linked list
                self.next = if let Some(mut next) = page.next() {
                    next.prev_insert(None);
                    Some(next)
                } else {
                    None
                };
                page.next_insert(None);
                Some(page)
            } else {
                // allocate new from buddy allocator
                let ptr = unsafe {
                    HEAP.buddy_allocator
                        .lock()
                        .alloc(Layout::array::<u8>(1 << self.order).unwrap())
                };
                debugln!("Buddy allocates {:#x}.", ptr as usize);
                let mut page = fetch_page(ptr as usize).unwrap();
                *page.order_mut() = self.order;
                page.free = LinkedList::new();
                page.make_slab();
                Some(page)
            }
        }

        // find the first free
        let mut page = self.curr.unwrap();
        let ptr = page.take_free().unwrap() as usize;
        if !page.is_free() {
            self.curr = None;
        }
        ptr
    }

    pub fn dealloc(&mut self, ptr: usize) {
        let mut page = fetch_page(ptr).unwrap();

        // the page is full previously
        if !page.is_free() {
            // the page is not inside next
            if let Some(mut next) = self.next {
                next.prev_insert(Some(page));
                page.next_insert(Some(next));
                page.prev_insert(None);
            }
            self.next = Some(page);
        }

        // insert a free object inside slab
        page.insert_free(ptr as *mut usize);

        // if the page is not in used, it should be deallocated
        if page.inuse() == 0 {
            if Some(page) != self.curr {
                if let Some(mut prev) = page.prev() {
                    // it's not the head of slabs
                    prev.next_insert(page.next());
                    if let Some(mut next) = page.next() {
                        next.prev_insert(Some(prev));
                    }
                } else {
                    // it's the head of slabs
                    if let Some(mut next) = page.next() {
                        next.prev_insert(None);
                    }
                    self.next = None;
                }
                page.prev_insert(None);
                page.next_insert(None);
                unsafe {
                    debugln!("Buddy deallocates {:#x}.", page.pa());
                    HEAP.buddy_allocator.lock().dealloc(
                        page.pa() as *mut u8,
                        Layout::array::<u8>(1 << self.order).unwrap(),
                    );
                }
            }
        }
    }
}

impl Cache {
    pub const fn empty() -> Cache {
        Cache {
            order: 0,
            curr: None,
            next: None,
        }
    }

    pub fn order_mut(&mut self) -> &mut usize {
        &mut self.order
    }
}
