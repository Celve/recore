mod cache;
mod heap;
pub mod slab_allocator;

use crate::config::KERNEL_HEAP_SIZE;

use self::heap::Heap;

#[link_section = ".data.heap"]
static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static HEAP: Heap = Heap::default();

pub fn init_heap() {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        let end = start + KERNEL_HEAP_SPACE.len();

        HEAP.init(start, end);
    }
}
