use core::cell::{RefCell, RefMut};

pub struct UpCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UpCell<T> {}

impl<T> UpCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.inner.borrow_mut()
    }
}
