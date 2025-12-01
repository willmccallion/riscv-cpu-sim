.PHONY: all clean run test

# Default: Build everything
all: build-software build-hardware

# 1. Build the OS (C/Asm) -> produces software/disk.img
build-software:
	@echo "--- Building OS & Userland ---"
	$(MAKE) -C software

# 2. Build the CPU (Rust) -> produces hardware/target/release/riscv-cpu
build-hardware:
	@echo "--- Building CPU Simulator ---"
	cd hardware && cargo build --release

# 3. Run them together
# Passes the disk image created by step 1 to the CPU created by step 2
run: all
	@echo "--- Booting RISC-V System ---"
	./hardware/target/release/riscv-cpu --disk software/disk.img

# Clean both
clean:
	$(MAKE) -C software clean
	cd hardware && cargo clean
