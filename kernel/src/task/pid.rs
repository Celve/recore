use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::sync::up::UpCell;

pub struct Pid(pub usize);

pub struct PidAllocator {
    curr: usize,
    recycled: Vec<usize>,
}

impl Pid {
    pub fn new(pid: usize) -> Self {
        Self(pid)
    }
}

impl Drop for Pid {
    fn drop(&mut self) {
        dealloc_pid(self);
    }
}

impl PidAllocator {
    pub fn new() -> Self {
        Self {
            curr: 0, // TODO: decide use 0 or 1 as init pid
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> Pid {
        Pid::new(if let Some(pid) = self.recycled.pop() {
            pid
        } else {
            let res = self.curr;
            self.curr += 1;
            res
        })
    }

    pub fn dealloc(&mut self, pid: &Pid) {
        self.recycled.push(pid.0);
    }
}

lazy_static! {
    pub static ref PID_ALLOCATOR: UpCell<PidAllocator> = UpCell::new(PidAllocator::new());
}

pub fn alloc_pid() -> Pid {
    PID_ALLOCATOR.borrow_mut().alloc()
}

pub fn dealloc_pid(pid: &Pid) {
    PID_ALLOCATOR.borrow_mut().dealloc(pid);
}
