TARGET = riscv64-elf
CC = $(TARGET)-gcc
LD = $(TARGET)-ld
OBJCOPY = $(TARGET)-objcopy

# Directories
BUILD_DIR = build
BIN_DIR   = bin
LIB_DIR   = libc
USER_DIR  = user
KERN_DIR  = kernel

# Flags
CFLAGS = -march=rv64g -mabi=lp64 -mcmodel=medany -ffreestanding -nostdlib -g -O0

# --- Kernel Build ---
KERN_CFLAGS = $(CFLAGS) -I$(KERN_DIR)/include

# 1. Find C files
KERN_SRC_DIRS = $(KERN_DIR) $(KERN_DIR)/drivers $(KERN_DIR)/lib $(KERN_DIR)/fs
KERN_C_SRCS = $(foreach dir, $(KERN_SRC_DIRS), $(wildcard $(dir)/*.c))

# 2. Define Objects
# IMPORTANT: We separate entry.o so we can put it first in the linker command
KERN_ENTRY_OBJ = $(BUILD_DIR)/kernel/entry.o
KERN_C_OBJS    = $(patsubst $(KERN_DIR)/%.c, $(BUILD_DIR)/kernel/%.o, $(KERN_C_SRCS))

KERN_BIN  = $(BUILD_DIR)/kernel.bin

# --- Libc & User ---
USER_CFLAGS = $(CFLAGS) -I$(LIB_DIR)

CRT0_OBJ   = $(BUILD_DIR)/libc/crt0.o
STDIO_OBJ  = $(BUILD_DIR)/libc/stdio.o
STDLIB_OBJ = $(BUILD_DIR)/libc/stdlib.o
LINKER_SCR = $(LIB_DIR)/user.ld

USER_C_SRCS = $(wildcard $(USER_DIR)/*.c)
USER_C_BINS = $(patsubst $(USER_DIR)/%.c, $(BIN_DIR)/%.bin, $(USER_C_SRCS))

USER_S_SRCS = $(wildcard $(USER_DIR)/*.s)
USER_S_BINS = $(patsubst $(USER_DIR)/%.s, $(BIN_DIR)/%.bin, $(USER_S_SRCS))

.PHONY: all clean dirs disk

all: dirs disk

dirs:
	@mkdir -p $(BUILD_DIR)/kernel/drivers \
	          $(BUILD_DIR)/kernel/lib \
	          $(BUILD_DIR)/kernel/fs \
	          $(BUILD_DIR)/libc \
	          $(BUILD_DIR)/user \
	          $(BIN_DIR)

# --- Compile Libc ---
$(CRT0_OBJ): $(LIB_DIR)/crt0.s
	@echo "  AS (Lib) $<"
	@$(CC) $(USER_CFLAGS) -c $< -o $@
$(STDIO_OBJ): $(LIB_DIR)/stdio.c
	@echo "  CC (Lib) $<"
	@$(CC) $(USER_CFLAGS) -c $< -o $@
$(STDLIB_OBJ): $(LIB_DIR)/stdlib.c
	@echo "  CC (Lib) $<"
	@$(CC) $(USER_CFLAGS) -c $< -o $@

# --- Compile Kernel ---
$(BUILD_DIR)/kernel/%.o: $(KERN_DIR)/%.c
	@echo "  CC (Kern) $<"
	@$(CC) $(KERN_CFLAGS) -c $< -o $@

$(BUILD_DIR)/kernel/entry.o: $(KERN_DIR)/entry.s
	@echo "  AS (Kern) $<"
	@$(CC) $(KERN_CFLAGS) -c $< -o $@

# LINKER STEP: Put KERN_ENTRY_OBJ first!
$(KERN_BIN): $(KERN_ENTRY_OBJ) $(KERN_C_OBJS)
	@echo "  LD (Kern) $@"
	@$(LD) -Ttext 0x80100000 $(KERN_ENTRY_OBJ) $(KERN_C_OBJS) -o $(BUILD_DIR)/kernel.elf
	@$(OBJCOPY) -O binary $(BUILD_DIR)/kernel.elf $@

# --- Compile User Apps ---
$(BIN_DIR)/%.bin: $(USER_DIR)/%.c $(CRT0_OBJ) $(STDIO_OBJ) $(STDLIB_OBJ)
	@echo "  CC (User) $<"
	@$(CC) $(USER_CFLAGS) -c $< -o $(BUILD_DIR)/user/$*.o
	@$(LD) -T $(LINKER_SCR) $(CRT0_OBJ) $(STDIO_OBJ) $(STDLIB_OBJ) $(BUILD_DIR)/user/$*.o -o $(BUILD_DIR)/user/$*.elf
	@$(OBJCOPY) -O binary $(BUILD_DIR)/user/$*.elf $@

$(BIN_DIR)/%.bin: $(USER_DIR)/%.s
	@echo "  AS (User) $<"
	@$(CC) $(USER_CFLAGS) -c $< -o $(BUILD_DIR)/user/$*.o
	@$(LD) -T $(LINKER_SCR) $(BUILD_DIR)/user/$*.o -o $(BUILD_DIR)/user/$*.elf
	@$(OBJCOPY) -O binary $(BUILD_DIR)/user/$*.elf $@

# --- Create Disk Image ---
disk: $(KERN_BIN) $(USER_C_BINS) $(USER_S_BINS)
	@echo "  PACKING disk.img"
	@python3 mkfs.py

clean:
	rm -rf $(BUILD_DIR) $(BIN_DIR) *.bin *.img
