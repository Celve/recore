# Allocator

There are three allocators inside this kernel: frame allocator, buddy allocator, and slub allocator (for some reason, it might be called the slab allocator later).

## Frame Allocator

Frame allocator is located in `kernel/src/mm/frame.rs`, which specializes in creating frames for page tables and user programs. It owns all memory from the end of kernel program to the `MEMORY_END`, which is defined in the `config.rs`.

The structure of frame allocator is like

```rust
/// kernel/src/mm/frame.rs
pub struct FrameAllocator {
    start: PhyPageNum,
    end: PhyPageNum,
    recycled: Vec<PhyPageNum>,
}
```

Hence, we only need to setup `start` and `end` for initialization.

### Allocation

For allocation, it gets the page when the `recycled` is not empty. Otherwise, it gets the one by adding `1` to `start`.

### Deallocation

The deallocation is done by adding the page number to `recycled`. The deallocation is automatically done by implementing the `Drop` trait for `Frame`, which implies the idea of RAII.

## Buddy allocator

Buddy allocator is the heap allocator in kernel program. To distinguish it from the slab allocator, its smallest allocation granularity is 4kb. It locates at `allocator/src/buddy_allocator.rs`. 

It needs a contiuous memory space to initialize. 

### Allocation

The buddy allocator contains a series of implicit linked lists, each of which corresponds to different size of the power of 2. 

For allocation, we identify the eligible linked list, trying to find one block of memory to allocate. If there is none, we need to traverse up the linked list to find an available block. Then, we split it up to the proper size for the request of allocation. 

### Deallocation

We simply put the block back to its linked list. If there is a adjacent one, we merge it and put it to the linked list in the next level. We repeat this process for the emergent block until there is no adjancent block for it. 

## Slab Allocator

Actually I implement the slub allocator. Generally speaking, slab always conflates with slub, and slab is more common in documents. Hence, I just call it slab here.

It locates in `kernel/src/heap/slab_allocator.rs`. Basically, it doesn't need any kind of initialization, because it gets pages from the buddy allocator.

### Allocation

The allocation of slab allocator is basically the allocation inside one page. Likewise, the granularity of slab allocation ranges from 1 byte to 4096 bytes, with the gradient of the power of 2. 

The basic structure of slab allocator is

```rust
/// kernel/src/heap/slab_allocator.rs
pub struct SlabAllocator {
    caches: [Spin<Cache>; PAGE_SIZE_BITS + 1],
}
```

The cache is the basic unit for slab allocator. As its definition here:

```rust
/// kernel/src/heap/cache.rs
#[derive(Clone, Copy)]
pub struct Cache {
    order: usize,
    curr: Option<PagePtr>,
    next: Option<PagePtr>,
}
```

As its name, it's a real cache for pages allocated from buddy allocator. The field `curr` contains only one page that is unfilled or half-filled. The field `next` represents a linked list of page, in which the pages are all half-filled.

The slab allocator fetches space from it to allocate. When there is no room for allocation inside the page, the `curr` is simply thrown away. The allocator then fetches a page from the linked list from `next` or by allocation of buddy allocator. 

### Deallocation

We use the pointer to locate the page it belongs to, whether it's a page inside the `next` linked list, a page that `curr` represents, or an already thrown-away full page. 

If it belongs to a full page, we need to add it back to the `next` linked list. 

When the join results in the occurence of a unfilled page, it should be deallocated by buddy allocator.