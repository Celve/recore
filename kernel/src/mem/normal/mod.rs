use crate::{config::MEMORY_END, mem::allocator::PageAllocator};

use self::allocator::FrameAllocator;

pub mod allocator;
pub mod page;

pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::default();

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    unsafe { FRAME_ALLOCATOR.init((ekernel as usize).into(), MEMORY_END.into()) };
}
