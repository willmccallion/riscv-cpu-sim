use crate::cpu::Cpu;
use crate::isa::abi;

pub fn wb_stage(cpu: &mut Cpu) {
    let wb = cpu.mem_wb;
    if cpu.trace {
        eprintln!("WB  pc={:#x} inst={:#010x}", wb.pc, wb.inst);
    }

    // Count retired instructions
    // We ignore bubbles (inst == 0x13 is NOP, 0 is bubble/flush)
    if wb.inst != 0x0000_0000 && wb.inst != 0x0000_0013 {
        cpu.stats.instructions_retired += 1;
    }

    if wb.ctrl.reg_write && wb.rd != 0 {
        let val = if wb.ctrl.mem_read {
            wb.load_data
        } else if wb.ctrl.jump {
            wb.pc.wrapping_add(4)
        } else {
            wb.alu
        };
        cpu.regs.write(wb.rd, val);
    }

    // Minimal ecall handling: a7=93 means exit with code in a0
    if wb.inst == 0x00000073 {
        // ECALL
        if cpu.regs.read(abi::A7) == 93 {
            cpu.exit_code = Some(cpu.regs.read(abi::A0));
        }
    }
}
