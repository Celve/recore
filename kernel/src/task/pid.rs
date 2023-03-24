use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

/// Data structure for pid to control allocation and deallocation.
pub struct Pid(pub usize);

/// A pid allocator that allocates pid from 1.
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
            curr: 1,
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
    pub static ref PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
}

pub fn alloc_pid() -> Pid {
    PID_ALLOCATOR.lock().alloc()
}

pub fn dealloc_pid(pid: &Pid) {
    PID_ALLOCATOR.lock().dealloc(pid);
}
