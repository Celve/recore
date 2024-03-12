use lazy_static::lazy_static;

use self::allocator::FrameAllocator;
use crate::{config::MEMORY_END, mem::allocator::PageAllocator};

pub mod allocator;
pub mod page;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::default();
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    unsafe { FRAME_ALLOCATOR.init((ekernel as usize).into(), MEMORY_END.into()) };
}
