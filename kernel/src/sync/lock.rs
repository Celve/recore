use alloc::sync::Arc;

pub trait Lock<T> {
    fn lock(&self) -> LockGuard<T>;
}

pub struct LockGuard<'a, T: 'a> {
    lock: Arc<dyn Lock<T>>,
    data: &'a mut T,
}
