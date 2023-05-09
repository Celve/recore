use crate::sync::basic::{BlockLock, SpinLock};

pub enum Lockable {
    SpinMutex(SpinLock),
    BlockMutex(BlockLock),
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

    pub fn is_locked(&self) -> bool {
        match self {
            Lockable::SpinMutex(mutex) => mutex.is_locked(),
            Lockable::BlockMutex(mutex) => mutex.is_locked(),
        }
    }
}
