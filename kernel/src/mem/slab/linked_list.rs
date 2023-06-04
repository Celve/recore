use core::ptr::null_mut;

#[derive(Clone, Copy, Debug)]
pub struct LinkedList {
    pub head: *mut usize,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
    pub const fn new() -> LinkedList {
        LinkedList { head: null_mut() }
    }

    /// Push an element to the front of the linked list.
    ///
    /// This function is unsafe because user has to guarantee that *node is a legal pointer.
    pub unsafe fn push_front(&mut self, node: *mut usize) {
        *node = self.head as usize;
        self.head = node;
    }

    /// Pop the first element of the linked list. Return `None` when the linked list is empty.
    pub fn pop_front(&mut self) -> Option<*mut usize> {
        return if !self.is_empty() {
            let result = self.head;
            self.head = unsafe { *self.head } as *mut usize;
            Some(result)
        } else {
            None
        };
    }

    /// Check whether the linked list is empty.
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,
            linked_list: self,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            linked_list: self,
        }
    }
}

pub struct Iter<'a> {
    curr: *mut usize,
    linked_list: &'a LinkedList,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = self.curr;
            self.curr = unsafe { *self.curr } as *mut usize;
            Some(result)
        } else {
            None
        }
    }
}

pub struct LinkedListInner {
    prev: *mut usize,
    curr: *mut usize,
}

impl LinkedListInner {
    pub fn as_ptr(&self) -> *mut usize {
        self.curr
    }

    pub fn pop(self) -> *mut usize {
        unsafe {
            *self.prev = *self.curr;
        }
        self.curr
    }
}

pub struct IterMut<'a> {
    prev: *mut usize,
    curr: *mut usize,
    linked_list: &'a mut LinkedList,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = LinkedListInner;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = LinkedListInner {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe { *self.curr } as *mut usize;
            Some(result)
        } else {
            None
        }
    }
}
