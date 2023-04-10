use core::mem::MaybeUninit;

use alloc::sync::Arc;

pub trait DiskManager: Send + Sync {
    fn read(&self, bid: usize, buf: &mut [u8]);
    fn write(&self, bid: usize, buf: &[u8]);
}
