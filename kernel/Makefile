# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
FS_IMG := ../fuse/fs.img
APPS := ../user/src/bin/*

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80000000

# Binutils
OBJCOPY := rust-objcopy --binary-architecture=riscv64
OBJDUMP := rust-objdump --arch-name=riscv64

# Disassembly
DISASM ?= -d

build: env $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fsimg: $(APPS)
	@cd ../user && make build && cd ../kernel
	@cd ../fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/

kernel:
	@cargo build --release

clean:
	@cargo clean

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios none \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios none -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -drive file=$(FS_IMG),if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 -s -S" && \
		tmux split-window -h "riscv64-linux-gnu-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@nvim $(DISASM_TMP)
	@rm $(DISASM_TMP)

.PHONY: build env kernel clean run debug fsimg
