/// A trait that support read from or write to a disk according to the given block id.
pub trait DiskManager: Send + Sync {
    fn read(&self, bid: usize, buf: &mut [u8]);
    fn write(&self, bid: usize, buf: &[u8]);
}
