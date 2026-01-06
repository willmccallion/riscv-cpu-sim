use clap::Parser;
use std::{fs, process};

mod cpu;
mod devices;
mod isa;
mod memory;
mod register_file;
mod stats;

use cpu::Cpu;
use devices::{Bus, VirtualDisk};
use memory::{BASE_ADDRESS, Memory};

// The Bootloader is "Firmware" (ROM).
const BIOS_BYTES: &[u8] = include_bytes!("../bootloader/boot.bin");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the disk image (contains Kernel + User Apps)
    #[arg(short, long, default_value = "disk.img")]
    disk: String,

    /// Enable instruction tracing
    #[arg(short, long)]
    trace: bool,
}

fn main() {
    let args = Args::parse();

    println!("RISC-V CPU Simulator v1.0");
    println!("-------------------------");

    let mut mem = Memory::new();
    mem.load(BIOS_BYTES, 0);

    println!("[*] Mounting Disk: {}", args.disk);
    let disk_data = fs::read(&args.disk).unwrap_or_else(|_| {
        eprintln!("Error: Could not read disk image '{}'", args.disk);
        process::exit(1);
    });

    let mut disk = VirtualDisk::new();
    disk.load(disk_data);

    let uart = devices::Uart::new();
    let bus = Bus::new(mem, uart, disk);
    let mut cpu = Cpu::new(BASE_ADDRESS, args.trace, bus);

    println!("[*] CPU Reset. Execution started.");

    loop {
        if let Err(e) = cpu.tick() {
            eprintln!("\n[!] FATAL TRAP: {}", e);
            cpu.dump_state();
            cpu.print_stats();
            process::exit(1);
        }

        if let Some(code) = cpu.take_exit() {
            cpu.print_stats();
            process::exit(code as i32);
        }
    }
}
