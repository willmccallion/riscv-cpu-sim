use crate::cpu::Cpu;
use crate::cpu::MEMORY_LATENCY;
use crate::cpu::pipeline::IFID;

pub fn fetch_stage(cpu: &mut Cpu) -> Result<(), String> {
    // Simulate I-Cache access
    if cpu.i_cache.access(cpu.pc) {
        cpu.stats.icache_hits += 1;
    } else {
        cpu.stats.icache_misses += 1;
        // Cache miss penalty
        cpu.stall_cycles += MEMORY_LATENCY;
    }

    let inst = cpu.read_inst(cpu.pc);
    if cpu.trace {
        eprintln!("IF  pc={:#x} inst={:#010x}", cpu.pc, inst);
    }
    cpu.if_id = IFID { pc: cpu.pc, inst };
    cpu.pc = cpu.pc.wrapping_add(4);
    Ok(())
}
