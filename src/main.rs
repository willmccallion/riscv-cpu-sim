use std::{env, fs, path::PathBuf};

mod cpu;
mod devices;
mod isa;
mod memory;
mod register_file;
mod stats;

use cpu::Cpu;
use devices::{Bus, VirtualDisk};
use memory::{BASE_ADDRESS, Memory};

const BIOS_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bootloader.bin"));
const KERNEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kernel.bin"));

fn main() {
    // Args: <program.s> [--trace]
    let mut trace = false;
    let mut asm_path: Option<PathBuf> = None;
    for arg in env::args().skip(1) {
        if arg == "--trace" {
            trace = true;
        } else if !arg.starts_with('-') {
            asm_path = Some(PathBuf::from(arg));
        }
    }
    if asm_path.is_none() {
        eprintln!("usage: cpu <program.s> [--trace]");
        std::process::exit(1);
    }

    let prog_bytes = match asm_path
        .as_ref()
        .unwrap()
        .extension()
        .and_then(|s| s.to_str())
    {
        Some("bin") => fs::read(asm_path.as_ref().unwrap()).expect("read .bin"),
        _ => {
            eprintln!("expected a flat binary (.bin). Hook your assembler here if needed.");
            std::process::exit(2);
        }
    };

    // Build RAM + devices + bus
    let mut mem = Memory::new();
    // BIOS at physical 0
    mem.load(BIOS_BYTES, 0);

    // Disk layout: [kernel (padded to 4KiB) | user program]
    let mut disk_img = KERNEL_BYTES.to_vec();
    disk_img.resize(4096, 0);
    disk_img.extend_from_slice(&prog_bytes);

    let mut disk = VirtualDisk::new();
    disk.load(disk_img);

    let uart = devices::Uart::new();
    let bus = Bus::new(mem, uart, disk);

    // CPU @ BASE_ADDRESS (BIOS runs from 0x8000_0000 in our flat map)
    let mut cpu = Cpu::new(BASE_ADDRESS, trace, bus);

    // Convention: pass kernel size in a0 so kernel knows user offset
    cpu.regs.write(10, 4096); // a0

    // Run until BIOS/kernel/user program calls an ecall 93 (exit) or we hit a guard
    let mut guard_cycles: u64 = 100_000_000;
    println!("-----------------------------\n");
    loop {
        if let Err(e) = cpu.tick() {
            eprintln!("TRAP: {e}");
            cpu.dump_state();
            cpu.print_stats(); // Print stats on error
            std::process::exit(1);
        }
        if cpu.take_exit().is_some() {
            let code = cpu.exit_code.unwrap_or(0) as i32;
            cpu.print_stats(); // Print stats on normal exit
            std::process::exit(code);
        }
        guard_cycles -= 1;
        if guard_cycles == 0 {
            eprintln!("Simulation guard hit");
            cpu.dump_state();
            cpu.print_stats(); // Print stats on timeout
            std::process::exit(3);
        }
    }
}
