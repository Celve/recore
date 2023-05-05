# Bootloader

The entry point of the whole kernel is `0x80000000`, which indicates the `_start()` function inside `kernel/src/main.rs`. 

Because the kernel support multi-threading, each hart would have its specified bootloader stack. In the `_start`, we correctly setup `sp` according to `mhartid`.

After that, we enter `rust_start()` inside `kernel/src/main.rs`, which configures the followings: 

1. `mstatus` and `mepc` for `mret` to the real main. Here, we need to set `mpp` in `mstatus` to supervisor, because privilege changed in `mret` according to it. 
2. `pmpaddr0` and `pmpcfg0` for physical memory protection. In this toy kernel, I didn't setup delicate protection on memory. Instead, I just allow all address to be accessed.
3. Save `mhartid` to `tp`, which is a CPU-specific register.
4. Initialize timer for machine mode timer interrupt, which is introduced later.
5. Delegate all interrupts and exceptions to supervisor mode by setting the `mideleg` and `medeleg` registers. 

After configurations, we enter the real main function `rust_main()` by using `mret`.
