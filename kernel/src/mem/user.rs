use core::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::Page;

/// One possible representation of metadata of page, when the page is used by user.
#[derive(Debug)]
pub struct UserPage {
    pa: AtomicUsize,
    cnt: AtomicUsize,
}

pub struct UserPagePtr {
    pa: usize,
}
