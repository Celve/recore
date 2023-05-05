# Page Table

The page table used in this kernel is SV39. To enable MMU in the supervisor mode, we only need to setup the `satp` to the physical page number of the root page of the page table with a `mode` field to indicate the type of page table it used. 
