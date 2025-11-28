use crate::cpu::Cpu;
use crate::cpu::MEMORY_LATENCY;
use crate::cpu::control::MemWidth;
use crate::cpu::pipeline::MEMWB;

pub fn mem_stage(cpu: &mut Cpu) -> Result<(), String> {
    let ex = cpu.ex_mem;
    if cpu.trace {
        eprintln!("MEM pc={:#x} inst={:#010x}", ex.pc, ex.inst);
    }

    let addr = ex.alu;
    let mut ld: u64 = 0;

    // Simulate D-Cache access if reading or writing
    if ex.ctrl.mem_read || ex.ctrl.mem_write {
        // Skip cache stats for IO regions (Disk/UART) to avoid skewing stats
        if addr < 0x9000_0000 {
            if cpu.d_cache.access(addr) {
                cpu.stats.dcache_hits += 1;
            } else {
                cpu.stats.dcache_misses += 1;
                // Cache miss penalty
                cpu.stall_cycles += MEMORY_LATENCY;
            }
        }
    }

    if ex.ctrl.mem_read {
        if (crate::devices::VIRTUAL_DISK_SIZE_ADDRESS
            ..crate::devices::VIRTUAL_DISK_SIZE_ADDRESS + 8)
            .contains(&addr)
            && cpu.trace
        {
            let w = match ex.ctrl.width {
                MemWidth::Byte => "u8",
                MemWidth::Half => "u16",
                MemWidth::Word => "u32",
                MemWidth::Double => "u64",
                _ => "?",
            };

            let peek = match (ex.ctrl.width, ex.ctrl.signed_load) {
                (MemWidth::Byte, _) => cpu.load_u8(addr) as u64,
                (MemWidth::Half, _) => cpu.load_u16(addr) as u64,
                (MemWidth::Word, _) => cpu.load_u32(addr) as u64,
                (MemWidth::Double, _) => cpu.load_u64(addr),
                _ => 0,
            };
            eprintln!(
                "DISK_SIZE READ @pc={:#x} addr={:#x} width={} -> {:#x} ({})",
                ex.pc, addr, w, peek, peek
            );
        }
        ld = match (ex.ctrl.width, ex.ctrl.signed_load) {
            (MemWidth::Byte, true) => (cpu.load_u8(addr) as i8) as i64 as u64,
            (MemWidth::Half, true) => (cpu.load_u16(addr) as i16) as i64 as u64,
            (MemWidth::Word, true) => (cpu.load_u32(addr) as i32) as i64 as u64,
            (MemWidth::Byte, false) => cpu.load_u8(addr) as u64,
            (MemWidth::Half, false) => cpu.load_u16(addr) as u64,
            (MemWidth::Word, false) => cpu.load_u32(addr) as u64,
            (MemWidth::Double, _) => cpu.load_u64(addr),
            _ => 0,
        };
    } else if ex.ctrl.mem_write {
        match ex.ctrl.width {
            MemWidth::Byte => cpu.store_u8(addr, ex.store_data as u8),
            MemWidth::Half => cpu.store_u16(addr, ex.store_data as u16),
            MemWidth::Word => cpu.store_u32(addr, ex.store_data as u32),
            MemWidth::Double => cpu.store_u64(addr, ex.store_data as u64),
            _ => {}
        }
    }

    cpu.mem_wb = MEMWB {
        pc: ex.pc,
        inst: ex.inst,
        rd: ex.rd,
        alu: ex.alu,
        load_data: ld,
        ctrl: ex.ctrl,
    };

    Ok(())
}
