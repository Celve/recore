use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use fs::disk::DiskManager;
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

use crate::{
    config::VIRTIO_BASE_ADDRESS,
    mm::{frame::Frame, page_table::KERNEL_PAGE_TABLE},
};

pub struct BlkDev {
    blk: Mutex<VirtIOBlk<'static, VirIoHal>>,
}

pub struct VirIoHal;

lazy_static! {
    pub static ref VIRT_IO_FRAMES: Mutex<BTreeMap<usize, Vec<Frame>>> = Mutex::new(BTreeMap::new());
}

impl DiskManager for BlkDev {
    fn read(&self, bid: usize, buf: &mut [u8]) {
        self.blk.lock().read_block(bid, buf).unwrap();
    }

    fn write(&self, bid: usize, buf: &[u8]) {
        self.blk.lock().write_block(bid, buf).unwrap();
    }
}

impl BlkDev {
    pub fn new() -> Self {
        unsafe {
            Self {
                blk: Mutex::new(
                    VirtIOBlk::new(&mut *(VIRTIO_BASE_ADDRESS as *mut VirtIOHeader)).unwrap(),
                ),
            }
        }
    }
}

impl Hal for VirIoHal {
    fn dma_alloc(pages: usize) -> virtio_drivers::PhysAddr {
        let frames: Vec<Frame> = (0..pages).map(|_| Frame::fresh()).collect();
        let ptr = frames[0].ppn().into();
        VIRT_IO_FRAMES.lock().insert(ptr, frames);
        ptr
    }

    fn dma_dealloc(paddr: virtio_drivers::PhysAddr, pages: usize) -> i32 {
        VIRT_IO_FRAMES.lock().remove(&paddr);
        0
    }

    fn phys_to_virt(paddr: virtio_drivers::PhysAddr) -> virtio_drivers::VirtAddr {
        paddr
    }

    fn virt_to_phys(vaddr: virtio_drivers::VirtAddr) -> virtio_drivers::PhysAddr {
        KERNEL_PAGE_TABLE.translate_va(vaddr.into()).unwrap().into()
    }
}
