use core::ptr::null_mut;

#[derive(Clone, Copy)]
pub struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    pub const fn new() -> LinkedList {
        LinkedList { head: null_mut() }
    }

    /// This function is unsafe because user has to guarantee that *node is a legal pointer.
    pub unsafe fn push_front(&mut self, node: *mut usize) {
        *node = self.head as usize;
        self.head = node;
    }

    pub fn pop_front(&mut self) -> Option<*mut usize> {
        return if !self.is_empty() {
            let result = self.head;
            self.head = unsafe { *self.head } as *mut usize;
            Some(result)
        } else {
            None
        };
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn iter(&self) -> LinkedListIter {
        LinkedListIter {
            curr: self.head,
            linked_list: self,
        }
    }

    pub fn iter_mut(&mut self) -> LinkedListIterMut {
        LinkedListIterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            linked_list: self,
        }
    }
}

pub struct LinkedListIter<'a> {
    curr: *mut usize,
    linked_list: &'a LinkedList,
}

impl<'a> Iterator for LinkedListIter<'a> {
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

pub struct LinkedListNode {
    prev: *mut usize,
    curr: *mut usize,
}

impl LinkedListNode {
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

pub struct LinkedListIterMut<'a> {
    prev: *mut usize,
    curr: *mut usize,
    linked_list: &'a mut LinkedList,
}

impl<'a> Iterator for LinkedListIterMut<'a> {
    type Item = LinkedListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = LinkedListNode {
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
