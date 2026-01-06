#!/usr/bin/env python3
import struct
import os
import sys

# Configuration
# Increased to 64KB to prevent truncation with -O0 builds
KERNEL_SIZE = 65536
OUTPUT_IMG = "disk.img"
BIN_DIR = "bin"
KERNEL_BIN = "build/kernel.bin"

def main():
    if not os.path.exists(KERNEL_BIN):
        print(f"Error: {KERNEL_BIN} not found. Build kernel first.")
        sys.exit(1)

    with open(KERNEL_BIN, "rb") as f:
        kernel_data = f.read()

    if len(kernel_data) > KERNEL_SIZE:
        print(f"Warning: Kernel too big ({len(kernel_data)} > {KERNEL_SIZE}). Truncating.")
        kernel_data = kernel_data[:KERNEL_SIZE]

    padding = b'\x00' * (KERNEL_SIZE - len(kernel_data))
    disk_data = bytearray(kernel_data + padding)

    files = []
    if os.path.exists(BIN_DIR):
        for fname in os.listdir(BIN_DIR):
            if fname.endswith(".bin"):
                path = os.path.join(BIN_DIR, fname)
                with open(path, "rb") as f:
                    content = f.read()
                    name_bytes = fname.replace(".bin", "").encode('utf-8')[:31]
                    name_bytes += b'\x00' * (32 - len(name_bytes))
                    files.append({
                        "name": name_bytes,
                        "content": content,
                        "size": len(content)
                    })

    file_count = len(files)
    disk_data.extend(struct.pack("<I", file_count))

    header_size = 40
    data_offset = KERNEL_SIZE + 4 + (file_count * header_size)

    for file in files:
        disk_data.extend(file["name"])
        disk_data.extend(struct.pack("<I", data_offset))
        disk_data.extend(struct.pack("<I", file["size"]))
        data_offset += file["size"]

    for file in files:
        disk_data.extend(file["content"])

    with open(OUTPUT_IMG, "wb") as f:
        f.write(disk_data)

    print(f"Success: Created {OUTPUT_IMG} with Kernel + {file_count} apps.")

if __name__ == "__main__":
    main()
