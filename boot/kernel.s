.option norvc
    .section .text
    .global _start

# MMIO constants
.set DISK_BASE,        0x90000000
.set DISK_SIZE_ADDR,   0x90000FF8
.set USER_LOAD_ADDR,   0x0000000080200000   # RAM
.set KERNEL_SP,        0x00000000801FF000   # RAM

#------------------------------------------------------------------------------
# _start
#
# Description:
#   The Kernel Entry Point.
#   1. Sets up the kernel stack.
#   2. Calculates the location and size of the user program on disk.
#   3. Copies the user program into RAM.
#   4. Configures sstatus to drop privilege to User mode.
#   5. Executes sret to jump to the user program.
#
# Args:
#   a0: The size of the kernel (passed from bootloader)
#
# Register Usage:
#   sp: Kernel Stack Pointer
#   s0: Disk source pointer (User program start)
#   s1: RAM destination pointer (User program load address)
#   s2: Byte count (User program size)
#   s3: User entry point (saved for sepc)
#   t0: Temporary for address/CSR access
#   t1: Temporary for data transfer/masking
#------------------------------------------------------------------------------
_start:
    # Stack
    li      sp, KERNEL_SP                     # sp <- Kernel Stack Pointer

    # Read disk size -> s2
    li      t0, DISK_SIZE_ADDR                # t0 <- Address of Disk Size Register
    ld      s2, 0(t0)                         # s2 <- Total disk size in bytes

    # s2 = user_bytes = disk_size - kernel_size(a0)
    sub     s2, s2, a0                        # s2 <- User program size

    # s0 = disk src = DISK_BASE + kernel_size
    li      s0, DISK_BASE                     # s0 <- Disk Base Address
    add     s0, s0, a0                        # s0 <- Start of user program on disk

    # s1 = dst RAM
    li      s1, USER_LOAD_ADDR                # s1 <- User Load Address (RAM)

    # s3 = user entry (save for sepc)
    add     s3, s1, zero                      # s3 <- User Entry Point

copy_loop:
    beq     s2, zero, copy_done               # if s2 == 0 goto copy_done
    lbu     t1, 0(s0)                         # t1 <- Byte from disk
    sb      t1, 0(s1)                         # Store byte to RAM
    addi    s0, s0, 1                         # s0 <- s0 + 1 (Next disk addr)
    addi    s1, s1, 1                         # s1 <- s1 + 1 (Next RAM addr)
    addi    s2, s2, -1                        # s2 <- s2 - 1 (Decrement count)
    j       copy_loop                         # goto copy_loop

copy_done:
    # set sepc = user entry
    csrrw   zero, sepc, s3                    # sepc <- User Entry Point

    csrr    t0, sstatus                       # t0 <- Current sstatus
    li      t1, ~(1 << 8)                     # t1 <- Mask to clear SPP bit (bit 8)
    and     t0, t0, t1                        # t0 <- sstatus & ~SPP (Set SPP to User)
    csrw    sstatus, t0                       # Update sstatus

    # go
    sret                                      # Return from exception (Jump to User)

hang:
    j       hang                              # Infinite loop
