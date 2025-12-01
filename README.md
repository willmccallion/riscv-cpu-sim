# RISC-V System Emulator & Microkernel

A cycle-accurate RISC-V (RV64IM) processor simulator written in Rust, capable of booting a custom C-based microkernel. The project features a 5-stage pipeline with architectural features like branch prediction, a 3-level cache hierarchy, and MMU simulation.

The project is organized into two distinct subsystems: the hardware simulator (Rust) and the software operating system (C/Assembly). The simulator loads the operating system and user programs via a custom virtual disk image.

## Features

### CPU Architecture
*   **ISA:** RV64IM (Integer + Multiply/Divide extensions).
*   **Pipeline:** 5-Stage (Fetch, Decode, Execute, Memory, Writeback) with hazard detection and forwarding.
*   **Branch Prediction:**
    *   GShare predictor with Global History Register (GHR).
    *   Branch Target Buffer (BTB).
    *   Return Address Stack (RAS) for function calls.
*   **Memory Hierarchy:**
    *   **L1 Cache:** Split Instruction (16KB) and Data (16KB), 4-way set associative.
    *   **L2 Cache:** Unified 128KB, 8-way set associative.
    *   **L3 Cache:** Unified 2MB, 16-way set associative.
    *   **MMU:** Sv39-style translation simulation with TLB latency penalties.
*   **Privilege Modes:** Machine (M-Mode), Supervisor (S-Mode), and User (U-Mode).

### Operating System (Microkernel)
*   **Bootloader:** Assembly-based firmware (`boot.s`) that loads the kernel from disk to RAM.
*   **Kernel:** C-based microkernel handling:
    *   UART I/O drivers.
    *   Virtual File System (VFS) parsing.
    *   ELF-like binary loading.
    *   Trap/Interrupt delegation and handling.
    *   `malloc`/`free` memory management.
*   **User Space:** A command-line shell (`sh`) capable of listing files and executing programs.

## Directory Structure

The repository is split into hardware and software directories:

*   `hardware/`: Contains the Rust source code for the CPU simulator and the Assembly source for the bootloader (firmware).
*   `software/`: Contains the C source code for the kernel, libc, and user-space programs.
*   `software/mkfs.py`: Python script used to pack compiled binaries into a disk image.
*   `Makefile`: Top-level build script that orchestrates the compilation of both subsystems.

## Requirements

1.  **Rust:** Latest stable Cargo.
2.  **RISC-V Toolchain:** `riscv64-elf-gcc` (or `riscv64-unknown-elf-gcc`).
3.  **Make:** GNU Make.
4.  **Python 3:** Required for packing the filesystem image.

## Building and Running

The project uses a top-level Makefile to handle the build process for both the C software and Rust hardware.

### 1. Build and Run
To compile the OS, pack the disk image, build the simulator, and boot the system:

```bash
make run
```

This command performs the following steps automatically:
1.  Compiles the Kernel and User programs using GCC.
2.  Runs `mkfs.py` to pack the binaries into `software/disk.img`.
3.  Compiles the Rust CPU simulator using Cargo.
4.  Executes the simulator, passing the disk image as an argument.

### 2. Clean Build
To remove all build artifacts (binaries, disk images, and Rust target files):

```bash
make clean
```

## Usage

Once the simulator starts, it will boot the kernel and drop you into a shell:

```text
root@riscv:~# help
Built-ins: ls, help, clear, exit
root@riscv:~# ls
-r-x       16200    life
-r-x       14500    sand
-r-x        4096    sort
root@riscv:~# life
(Runs Conway's Game of Life)
```

## Memory Map

| Address Range | Description | Privilege |
| :--- | :--- | :--- |
| `0x1000_0000` | UART I/O (Byte-wise) | RW |
| `0x8000_0000` | Bootloader Entry (M-Mode) | RX |
| `0x8010_0000` | Kernel Base (S-Mode) | RWX |
| `0x8020_0000` | User Program Load Address | RWX |
| `0x9000_0000` | Virtual Disk (Memory Mapped) | R |
| `0x9000_0FF8` | Virtual Disk Size Register | R |

## Included Demo Programs

The system supports standard C libraries (via `stdio.h`/`stdlib.h` shims) and includes several demos:

*   **life:** Conway's Game of Life visualization.
*   **sand:** Falling sand physics simulation.
*   **maze:** A* pathfinding algorithm solving a generated maze.
*   **mandelbrot:** Fixed-point arithmetic Mandelbrot set renderer.
*   **sort:** Quick Sort and Merge Sort benchmarks.
*   **fib:** Recursive Fibonacci calculation.
*   **chess:** A basic chess engine.

## Statistics

Upon exit (typing `exit` in the shell), the simulator prints detailed execution statistics:

*   **IPC (Instructions Per Cycle)**
*   **Branch Prediction Accuracy** (GShare/BTB performance)
*   **Cache Hit/Miss Rates** (L1, L2, L3)
*   **Pipeline Stalls** (Breakdown of Memory vs Control vs Data hazards)
