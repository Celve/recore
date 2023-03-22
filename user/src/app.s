	.align 3
	.section .data
	.global _num_apps
_num_apps:
	.quad 1
	.quad app_0_start
	.quad app_0_end

	.global _app_names
_app_names:
	.string "hello_world"

	.section .data
	.global app_0_start
	.global app_0_end
	.align 3
app_0_start:
	.incbin "../user/target/riscv64gc-unknown-none-elf/release/hello_world"
app_0_end:


