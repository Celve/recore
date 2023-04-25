use super::linked_list::LinkedList;
use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::cmp::{max, min};
use core::mem::size_of;
use spin::{Spin, SpinGuard};

const BUDDY_ALLOCATOR_LEVEL: usize = 32;

pub struct BuddyAllocator {
    pub allocator: Spin<BuddyAllocatorInner>,
}

pub struct BuddyAllocatorInner {
    /// This array maintains lists of free spaces in different levels.
    /// Its index corresponds to the power of size.
    free_lists: [LinkedList; BUDDY_ALLOCATOR_LEVEL],

    /// Granularity is used for the minimum memory space that it can allocate.
    gran: usize,

    // statistics
    user: usize,
    allocated: usize,
    total: usize,
}

impl BuddyAllocator {
    pub const fn empty(gran: usize) -> Self {
        Self {
            allocator: Spin::new(BuddyAllocatorInner::empty(gran)),
        }
    }

    pub fn lock(&self) -> SpinGuard<BuddyAllocatorInner> {
        self.allocator.lock()
    }

    pub unsafe fn add_segment(&self, start: usize, end: usize) {
        self.allocator.lock().add_segment(start, end);
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.allocator.lock().dealloc(ptr, layout);
    }
}

impl BuddyAllocatorInner {
    pub const fn empty(gran: usize) -> Self {
        Self {
            free_lists: [LinkedList::new(); BUDDY_ALLOCATOR_LEVEL],
            user: 0,
            allocated: 0,
            total: 0,
            gran: max(gran, size_of::<usize>()),
        }
    }

    /// Create a buddy allocator with a range of memory [start, end).
    ///
    /// This function is unsafe because user has to make sure that all segments provided are not intersected and are available from memory.
    pub unsafe fn new(gran: usize, start: usize, end: usize) -> Self {
        let mut allocator = Self::empty(gran);
        allocator.add_segment(start, end);
        allocator
    }

    /// Make a range of memory [start, end) to become available for buddy allocator.
    ///
    /// This function is unsafe because user has to make sure that all segments provided are not intersected and are available from memory.
    pub unsafe fn add_segment(&mut self, mut start: usize, mut end: usize) {
        start = (start + self.gran - 1) & (!self.gran + 1);
        end = end & (!self.gran + 1);
        self.total += end - start;

        while start < end {
            let level = (end - start).trailing_zeros() as usize;
            self.free_lists[level].push_front(start as *mut usize);
            start += 1 << level;
        }
    }

    /// Allocate a range of memory according to the given layout.
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = self.calculate_size(&layout);
        let level = size.trailing_zeros() as usize;
        for i in level..self.free_lists.len() {
            if !self.free_lists[i].is_empty() {
                // split or no split to find a proper piece
                self.split(level, i);
                let result = self.free_lists[level]
                    .pop_front()
                    .expect("[buddy_allocator] Expect non-empty free list.");

                // maintain statistics
                self.user += layout.size();
                self.allocated += size;
                return result as *mut u8;
            }
        }
        panic!(
            "[buddy_allocator] Unable to allocate more space for size {}.",
            size
        );
    }

    /// Deallocate memory according to the address provided.
    ///
    /// It's unsafe because the address given should be the one that buddy allocator provided, otherwise some fatal error might occur.s
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = self.calculate_size(&layout);
        let level = size.trailing_zeros() as usize;
        self.merge(level, ptr);
    }

    /// Split from level start to level end.
    fn split(&mut self, start: usize, end: usize) {
        for i in (start..end).rev() {
            let ptr = self.free_lists[i + 1]
                .pop_front()
                .expect("[buddy_allocator] Expect non-empty free list.");

            // split into two in the i level
            unsafe {
                self.free_lists[i].push_front((ptr as usize + (1 << i)) as *mut usize);
                self.free_lists[i].push_front(ptr);
            }
        }
    }

    /// Merge from level min with newly added addr.
    fn merge(&mut self, start: usize, ptr: *mut u8) {
        let mut curr = ptr as usize;
        for i in start..self.free_lists.len() {
            let buddy = curr ^ (1 << i);
            let target = self.free_lists[i]
                .iter_mut()
                .find(|node| node.as_ptr() as usize == buddy);

            if let Some(node) = target {
                node.pop();
                curr = min(curr, buddy);
            } else {
                unsafe {
                    self.free_lists[i].push_front(curr as *mut usize);
                }
                break;
            }
        }
    }

    /// Calculate the supposed size with layout and size.
    fn calculate_size(&self, layout: &Layout) -> usize {
        return max(
            layout.size().next_power_of_two(),
            max(layout.align(), self.gran),
        );
    }
}
