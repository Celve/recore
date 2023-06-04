use crate::{
    mem::slab::{page::SlabPage, SLAB_MEM_SECTION},
    mm::address::PhyPageNum,
};

use super::page::SlabPageGuard;

#[derive(Clone, Copy)]
pub struct Cache {
    order: usize,
    curr: PhyPageNum,
    next: PhyPageNum,
}

impl Cache {
    pub fn alloc(&mut self) -> usize {
        // if the curr is empty
        if self.curr.is_null() {
            self.curr = if !self.next.is_null() {
                let mut page = SlabPageGuard::new(self.next);
                // find from existed, adjust linked list
                self.next = if !page.next.is_null() {
                    let mut next = SlabPageGuard::new(page.next);
                    next.prev = PhyPageNum::null();
                    next.ppn
                } else {
                    PhyPageNum::null()
                };
                page.next = PhyPageNum::null();
                page.ppn
            } else {
                // allocate new from buddy allocator
                let ptr = usize::from(unsafe { SLAB_MEM_SECTION.alloc() });
                debugln!("Buddy allocates {:#x}.", ptr);
                SlabPage::alloc(ptr, self.order as u8);
                ptr.into()
            };
        }

        // find the first free
        let mut page = SlabPageGuard::new(self.curr);
        let ptr = page.take_slot().unwrap() as usize;
        if !page.is_available() {
            self.curr = PhyPageNum::null();
        }
        ptr
    }

    /// This function is unsafe. Things would become out of control when ptr is invalid.
    pub unsafe fn dealloc(&mut self, ptr: usize) {
        let mut page = SlabPageGuard::new(ptr.into());

        // the page is full previously
        if !page.is_available() {
            // the page is not inside next
            if !self.next.is_null() {
                let mut next = SlabPageGuard::new(self.next);
                next.prev = page.ppn;
                page.next = next.ppn;
                page.prev = PhyPageNum::null();
            }
            self.next = page.ppn;
        }

        // insert a free object inside slab
        page.return_slot(ptr as *mut usize);

        // if the page is not in used, it should be deallocated
        if page.inuse == 0 && self.curr != page.ppn {
            if !page.prev.is_null() {
                // it's not the head of slabs
                let mut prev = SlabPageGuard::new(page.prev);
                prev.next = page.next;
            } else {
                // it's the head of slabs
                self.next = PhyPageNum::null();
            }
            if !page.next.is_null() {
                let mut next = SlabPageGuard::new(page.next);
                next.prev = page.prev;
            }
            page.prev = PhyPageNum::null();
            page.next = PhyPageNum::null();
            debugln!("Buddy deallocates {:#x}.", usize::from(page.ppn));
            SLAB_MEM_SECTION.dealloc(page.ppn.into());
        }
    }
}

impl Cache {
    pub const fn empty() -> Cache {
        Cache {
            order: 0,
            curr: PhyPageNum::null(),
            next: PhyPageNum::null(),
        }
    }

    pub fn order_mut(&mut self) -> &mut usize {
        &mut self.order
    }
}
