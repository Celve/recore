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

impl UserPage {
    pub fn new(pa: usize) -> UserPagePtr {
        let user_page = Self {
            pa: AtomicUsize::new(pa),
            cnt: AtomicUsize::new(0),
        };
        unsafe {
            *Page::from_addr_mut(pa) = Page::User(user_page);
        }
        UserPagePtr { pa }
    }
}

impl UserPagePtr {
    pub fn new(pa: usize) -> UserPagePtr {
        let user_page = Page::from_addr_mut(pa).as_user_mut();
        user_page.cnt.fetch_add(1, Ordering::SeqCst);
        Self { pa }
    }
}

impl Deref for UserPagePtr {
    type Target = UserPage;

    fn deref(&self) -> &Self::Target {
        Page::from_addr(self.pa).as_user()
    }
}

impl DerefMut for UserPagePtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Page::from_addr_mut(self.pa).as_user_mut()
    }
}

impl Drop for UserPagePtr {
    fn drop(&mut self) {
        let user_page = Page::from_addr_mut(self.pa).as_user_mut();
        user_page.cnt.fetch_sub(1, Ordering::SeqCst);
    }
}
