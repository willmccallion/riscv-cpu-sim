# This Makefile builds the bootloader, the kernel, and all user programs.

TARGET = riscv64-elf
CC = $(TARGET)-gcc
LD = $(TARGET)-ld
OBJCOPY = $(TARGET)-objcopy

# Intermediate files (.o, .elf) go into build/
# Final binaries (.bin) go into bin/
BUILD_DIR = build
BIN_DIR   = bin

# Build Flags
#
# -march=rv64g: Target RISC-V 64-bit with general-purpose extensions.
# -mcmodel=medany: Medium-any code model, suitable for position-independent code.
# -ffreestanding: No standard library.
# -nostdlib: Don't link against the standard library.
# -g: Include debug symbols.
# -O0: No optimization, makes debugging easier.
CFLAGS = -march=rv64g -mcmodel=medany -ffreestanding -nostdlib -g -O0

# Find all source files for each component.
USER_SRCS_C    = $(wildcard examples/*.c)
USER_SRCS_S    = $(wildcard examples/*.s)

# Generate paths for intermediate object files in the BUILD_DIR.
USER_OBJS_C    = $(patsubst examples/%.c, $(BUILD_DIR)/%.o, $(USER_SRCS_C))
USER_OBJS_S    = $(patsubst examples/%.s, $(BUILD_DIR)/%.o, $(USER_SRCS_S))

# Define the final output files that will be placed in the BIN_DIR.
USER_BINS      = $(patsubst $(BUILD_DIR)/%.o, $(BIN_DIR)/%.bin, $(USER_OBJS_C) $(USER_OBJS_S))

# The default rule. `make` or `make all` builds everything.
.PHONY: all
all: $(USER_BINS)

# Rule to run the VM with the shell program.
# Depends on `all` to ensure everything is built first.
.PHONY: run
run: all
	@echo "  RUN     cargo run -- bin/shell.bin"
	@cargo run -- bin/shell.bin

# Pattern rule to build any user program binary from its object file.
$(BIN_DIR)/%.bin: $(BUILD_DIR)/%.o
	@echo "  LD      $(BUILD_DIR)/$*.elf"
	@mkdir -p $(@D)
	@$(LD) -T examples/user.ld $< -o $(BUILD_DIR)/$*.elf
	@echo "  OBJCOPY $@"
	@$(OBJCOPY) -O binary $(BUILD_DIR)/$*.elf $@

# Rule to compile/assemble user program sources.
$(BUILD_DIR)/%.o: examples/%.c
	@echo "  CC      $<"
	@mkdir -p $(@D)
	@$(CC) -c $(CFLAGS) $< -o $@

$(BUILD_DIR)/%.o: examples/%.s
	@echo "  AS      $<"
	@mkdir -p $(@D)
	@$(CC) -c $(CFLAGS) $< -o $@

.PHONY: clean
clean:
	@echo "Cleaning up build directories..."
	@rm -rf $(BUILD_DIR) $(BIN_DIR)
