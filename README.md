# RISC-V CPU Simulator

This is a simulator for a RISC-V processor written in Rust. It implements a 5-stage pipeline (Fetch, Decode, Execute, Memory, Writeback) and supports the RV64I instruction set.

## Features

*   **5-Stage Pipeline:** Simulates the standard RISC-V pipeline stages.
*   **Hazard Handling:** Implements data forwarding and load-use stalls.
*   **System Support:** Supports Machine and Supervisor privilege modes, CSRs, and exception handling.
*   **I/O:** Includes a UART implementation for text output and a virtual disk for loading programs.
*   **Tracing:** Optional flag to print the state of the pipeline at every cycle.

## Requirements

*   Rust (Cargo)
*   A RISC-V toolchain (specifically `riscv64-elf-gcc` and related tools).
*   Make

## Usage

### Option 1: Using Make (Recommended)

This project includes a Makefile to automate the build process.

To compile all examples and the simulator:
```bash
make
```

To compile and immediately run the default shell program:
```bash
make run
```

To clean up build artifacts:
```bash
make clean
```

### Option 2: Manual Compilation

If you prefer to compile manually, you must assemble your code, link it using the provided linker script, and convert it to a raw binary.

1.  **Assemble/Compile:**
    ```bash
    riscv64-elf-gcc -march=rv64g -mcmodel=medany -ffreestanding -nostdlib -g -O0 -c examples/fib.s -o build/fib.o
    ```

2.  **Link:**
    ```bash
    riscv64-elf-ld -T examples/user.ld build/fib.o -o build/fib.elf
    ```

3.  **Convert to Binary:**
    ```bash
    riscv64-elf-objcopy -O binary build/fib.elf bin/fib.bin
    ```

4.  **Run:**
    ```bash
    cargo run --release -- bin/fib.bin
    ```

## Memory Map

*   **UART:** `0x1000_0000`
*   **RAM Base:** `0x8000_0000`
*   **Virtual Disk:** `0x9000_0000`
