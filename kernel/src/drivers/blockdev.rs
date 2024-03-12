use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{collections::BTreeMap, vec::Vec};
use lazy_static::lazy_static;
use simplefs::disk::DiskManager;
use virtio_drivers::{BlkResp, Hal, RespStatus, VirtIOBlk, VirtIOHeader};

use crate::{
    config::VIRT_IO_HEADER,
    mem::normal::page::NormalPageHandle,
    mm::page_table::KERNEL_PAGE_TABLE,
    sync::{condvar::Condvar, mcs::Mcs},
};

pub struct BlkDev {
    blk: Mcs<VirtIOBlk<'static, VirIoHal>>,
    non_blocking: AtomicBool,
    condvars: Vec<Condvar>,
}

pub struct VirIoHal;

lazy_static! {
    pub static ref VIRT_IO_FRAMES: Mcs<BTreeMap<usize, Vec<NormalPageHandle>>> =
        Mcs::new(BTreeMap::new());
}

impl DiskManager for BlkDev {
    /// Read the block from the block device.
    ///
    /// When non-blocking is enabled, this function might yield.
    fn read(&self, bid: usize, buf: &mut [u8]) {
        if !self.non_blocking.load(Ordering::Acquire) {
            self.blk.lock().read_block(bid, buf).unwrap();
        } else {
            let mut guard = self.blk.lock();
            let mut resp = BlkResp::default();
            let token = unsafe { guard.read_block_nb(bid, buf, &mut resp).unwrap() };
            let condvar = &self.condvars[token as usize];
            condvar.wait_mcs(guard); // suspend until read is done
            assert_eq!(resp.status(), RespStatus::Ok);
        }
    }

    /// Write the block to the block device.
    ///
    /// When non-blocking is enabled, this function might yield.
    fn write(&self, bid: usize, buf: &[u8]) {
        if !self.non_blocking.load(Ordering::Acquire) {
            self.blk.lock().write_block(bid, buf).unwrap();
        } else {
            let mut guard = self.blk.lock();
            let mut resp = BlkResp::default();
            let token = unsafe { guard.write_block_nb(bid, buf, &mut resp).unwrap() };
            let condvar = &self.condvars[token as usize];
            condvar.wait_mcs(guard); // suspend until read is done
            assert_eq!(resp.status(), RespStatus::Ok);
        }
    }
}

impl BlkDev {
    pub fn handle_irq(&self) {
        let mut guard = self.blk.lock();
        while let Ok(token) = guard.pop_used() {
            let condvar = &self.condvars[token as usize];
            condvar.notify_one(); // there is only that is waiting, which should be promised by the driver
        }
    }

    pub fn enable_non_blocking(&self) {
        self.non_blocking.store(true, Ordering::Release);
    }
}

impl BlkDev {
    pub fn new() -> Self {
        let blk = unsafe { VirtIOBlk::new(&mut *(VIRT_IO_HEADER as *mut VirtIOHeader)).unwrap() };
        let mut condvars = Vec::new();
        let num_channel = blk.virt_queue_size();
        (0..num_channel).for_each(|_| condvars.push(Condvar::new()));

        Self {
            blk: Mcs::new(blk),
            non_blocking: AtomicBool::new(false),
            condvars,
        }
    }
}

impl Hal for VirIoHal {
    fn dma_alloc(pages: usize) -> virtio_drivers::PhysAddr {
        let frames: Vec<NormalPageHandle> = (0..pages).map(|_| NormalPageHandle::new()).collect();
        let ptr = frames[0].ppn.into();
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
