use core::sync::atomic::AtomicUsize;

/// One possible representation of metadata of page, when the page is not used.
#[derive(Debug)]
pub struct EmptyPage {
    pa: AtomicUsize,
}

impl EmptyPage {
    pub fn new(pa: usize) -> EmptyPage {
        EmptyPage {
            pa: AtomicUsize::new(pa),
        }
    }
}
