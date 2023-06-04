use core::alloc::Layout;

use crate::mem::slab::{
    page::{SlabPage, SlabPagePtr},
    SLAB_MEM_SECTION,
};

#[derive(Clone, Copy)]
pub struct Cache {
    order: usize,
    curr: SlabPagePtr,
    next: SlabPagePtr,
}

impl Cache {
    pub fn alloc(&mut self) -> usize {
        // if the curr is empty
        if self.curr.is_null() {
            self.curr = if !self.next.is_null() {
                let mut page = self.next;
                // find from existed, adjust linked list
                self.next = if !page.next.is_null() {
                    let mut next = page.next;
                    next.prev = SlabPagePtr::null();
                    next
                } else {
                    SlabPagePtr::null()
                };
                page.next = SlabPagePtr::null();
                page
            } else {
                // allocate new from buddy allocator
                let ptr = usize::from(unsafe { SLAB_MEM_SECTION.alloc() });
                debugln!("Buddy allocates {:#x}.", ptr);
                SlabPage::alloc(ptr, self.order as u8)
            };
        }

        // find the first free
        let mut page = self.curr;
        let ptr = page.take_slot().unwrap() as usize;
        if !page.is_available() {
            self.curr = SlabPagePtr::null();
        }
        ptr
    }

    /// This function is unsafe. Things would become out of control when ptr is invalid.
    pub unsafe fn dealloc(&mut self, ptr: usize) {
        let mut page = SlabPagePtr::new(ptr);

        // the page is full previously
        if !page.is_available() {
            // the page is not inside next
            if !self.next.is_null() {
                let mut next = self.next;
                next.prev = page;
                page.next = next;
                page.prev = SlabPagePtr::null();
            }
            self.next = page;
        }

        // insert a free object inside slab
        page.return_slot(ptr as *mut usize);

        // if the page is not in used, it should be deallocated
        if page.inuse == 0 && self.curr != page {
            if !page.prev.is_null() {
                // it's not the head of slabs
                let mut prev = page.prev;
                prev.next = page.next;
            } else {
                // it's the head of slabs
                self.next = SlabPagePtr::null();
            }
            if !page.next.is_null() {
                let mut next = page.next;
                next.prev = page.prev;
            }
            page.prev = SlabPagePtr::null();
            page.next = SlabPagePtr::null();
            debugln!("Buddy deallocates {:#x}.", page.pa());
            SLAB_MEM_SECTION.dealloc(page.pa().into());
        }
    }
}

impl Cache {
    pub const fn empty() -> Cache {
        Cache {
            order: 0,
            curr: SlabPagePtr::null(),
            next: SlabPagePtr::null(),
        }
    }

    pub fn order_mut(&mut self) -> &mut usize {
        &mut self.order
    }
}
