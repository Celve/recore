use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use alloc::boxed::Box;

#[derive(Default)]
pub struct Mcs<T> {
    last: AtomicPtr<McsNode>,
    data: UnsafeCell<T>,
}

pub struct McsGuard<'a, T: 'a> {
    mcs: &'a Mcs<T>,
    node: *mut McsNode,
    data: &'a mut T,
}

pub struct McsNode {
    next: AtomicPtr<McsNode>,
    locked: AtomicBool,
}

unsafe impl<T: Send> Sync for Mcs<T> {}
unsafe impl<T: Send> Send for Mcs<T> {}

impl<T> Mcs<T> {
    pub fn new(data: T) -> Self {
        Self {
            last: AtomicPtr::new(core::ptr::null_mut()),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> McsGuard<T> {
        let node = Box::into_raw(Box::new(McsNode::new(
            AtomicPtr::new(core::ptr::null_mut()),
            AtomicBool::new(true),
        )));
        let prev = self.last.swap(node, Ordering::AcqRel);
        if prev.is_null() {
            // it might be meaningless to modify the `locked`
            McsGuard::new(self, node, unsafe { &mut *self.data.get() })
        } else {
            unsafe {
                (*prev).next.store(node, Ordering::Release);
            }
            while unsafe { (*node).locked.load(Ordering::Acquire) } {}
            McsGuard::new(self, node, unsafe { &mut *self.data.get() })
        }
    }
}

impl<'a, T: 'a> McsGuard<'a, T> {
    pub fn new(mcs: &'a Mcs<T>, node: *mut McsNode, data: &'a mut T) -> Self {
        Self { mcs, node, data }
    }

    pub fn mcs(&self) -> &'a Mcs<T> {
        self.mcs
    }
}

impl<T> Deref for McsGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<T> DerefMut for McsGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T: 'a> Drop for McsGuard<'a, T> {
    fn drop(&mut self) {
        let node_ptr = self.node;
        let node = unsafe { Box::from_raw(node_ptr) };
        let mut next_ptr = node.next.load(Ordering::Acquire);
        if next_ptr == core::ptr::null_mut() {
            if self
                .mcs
                .last
                .compare_exchange(
                    node_ptr,
                    core::ptr::null_mut(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
            {
                loop {
                    next_ptr = node.next.load(Ordering::Acquire);
                    if next_ptr != core::ptr::null_mut() {
                        break;
                    }
                    core::hint::spin_loop();
                }
                unsafe {
                    (*next_ptr).locked.store(false, Ordering::Release);
                }
            }
        } else {
            unsafe {
                (*next_ptr).locked.store(false, Ordering::Release);
            }
        }
    }
}

impl McsNode {
    pub fn new(next: AtomicPtr<McsNode>, locked: AtomicBool) -> Self {
        Self { next, locked }
    }
}
