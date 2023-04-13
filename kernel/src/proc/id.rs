use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::mutex::Mutex;

pub struct Id {
    id: usize,
    allocator: Arc<IdAllocator>,
}

pub struct IdAllocator {
    inner: Mutex<IdAllocatorInner>,
}

#[derive(Clone)]
pub struct IdAllocatorInner {
    next_id: usize,
    recycled: Vec<usize>,
}

lazy_static! {
    pub static ref PID_ALLOCATOR: Arc<IdAllocator> = Arc::new(IdAllocator::new());
    pub static ref GID_ALLOCATOR: Arc<IdAllocator> = Arc::new(IdAllocator::new());
}

impl IdAllocatorInner {
    fn new() -> Self {
        Self {
            next_id: 1,
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    pub fn dealloc(&mut self, id: usize) {
        self.recycled.push(id);
    }
}

impl IdAllocator {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(IdAllocatorInner::new()),
        }
    }

    pub fn alloc(self: &Arc<Self>) -> Arc<Id> {
        Id::new(self.inner.lock().alloc(), self.clone())
    }

    fn dealloc(&self, id: &Id) {
        self.inner.lock().dealloc(id.id)
    }
}

impl Clone for IdAllocator {
    fn clone(&self) -> Self {
        Self {
            inner: Mutex::new(self.inner.lock().clone()),
        }
    }
}

impl Id {
    fn new(id: usize, allocator: Arc<IdAllocator>) -> Arc<Self> {
        Arc::new(Self { id, allocator })
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl Drop for Id {
    fn drop(&mut self) {
        self.allocator.dealloc(&self);
    }
}

impl From<Id> for usize {
    fn from(value: Id) -> Self {
        value.id
    }
}
