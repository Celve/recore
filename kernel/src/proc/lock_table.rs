use alloc::{sync::Arc, vec::Vec};

use crate::sync::mutex::{BlockMutex, SpinMutex};

pub enum Lockable {
    SpinMutex(SpinMutex),
    BlockMutex(BlockMutex),
}

pub struct LockTable {
    locks: Vec<Option<Arc<Lockable>>>,
}

impl Lockable {
    pub fn lock(&self) {
        match self {
            Lockable::SpinMutex(mutex) => mutex.lock(),
            Lockable::BlockMutex(mutex) => mutex.lock(),
        }
    }

    pub fn unlock(&self) {
        match self {
            Lockable::SpinMutex(mutex) => mutex.unlock(),
            Lockable::BlockMutex(mutex) => mutex.unlock(),
        }
    }
}

impl LockTable {
    pub fn new() -> Self {
        Self { locks: Vec::new() }
    }

    pub fn alloc(&mut self, blocked: bool) -> usize {
        let lock = Arc::new(if blocked {
            Lockable::BlockMutex(BlockMutex::new())
        } else {
            Lockable::SpinMutex(SpinMutex::new())
        });
        let pos = self.locks.iter().position(|mutex| mutex.is_none());
        if let Some(pos) = pos {
            self.locks[pos] = Some(lock);
            pos
        } else {
            self.locks.push(Some(lock));
            self.locks.len() - 1
        }
    }

    pub fn dealloc(&mut self, id: usize) {
        self.locks[id].take();
    }

    pub fn get(&self, id: usize) -> Option<Arc<Lockable>> {
        self.locks[id].clone()
    }

    pub fn len(&self) -> usize {
        self.locks.len() - self.locks.iter().filter(|mutex| mutex.is_none()).count()
    }
}
