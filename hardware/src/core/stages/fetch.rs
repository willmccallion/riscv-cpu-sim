use crate::core::Cpu;
use crate::core::pipeline::{IfId, IfIdEntry};
use crate::core::types::{AccessType, TranslationResult, Trap, VirtAddr};
use crate::isa::{abi, opcodes};

pub fn fetch_stage(cpu: &mut Cpu) -> Result<(), String> {
    let mut fetched = Vec::new();
    let mut current_pc = cpu.pc;

    for _ in 0..cpu.pipeline_width {
        if current_pc % 4 != 0 {
            if fetched.is_empty() {
                return Err(format!(
                    "{:?}",
                    Trap::InstructionAddressMisaligned(current_pc)
                ));
            }
            break;
        }

        let TranslationResult {
            paddr,
            cycles,
            trap,
        } = cpu.translate(VirtAddr::new(current_pc), AccessType::Fetch);
        cpu.stall_cycles += cycles;

        if let Some(trap_msg) = trap {
            if fetched.is_empty() {
                return Err(format!("{:?}", trap_msg));
            }
            break;
        }

        let latency = cpu.simulate_memory_access(paddr, AccessType::Fetch);
        cpu.stall_cycles += latency;

        let inst = cpu.bus.bus.read_u32(paddr.val());

        if cpu.trace {
            eprintln!("IF  pc={:#x} inst={:#010x}", current_pc, inst);
        }

        let opcode = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let mut next_pc_calc = current_pc.wrapping_add(4);
        let mut pred_taken = false;
        let mut pred_target = 0;
        let mut stop_fetch = false;

        if opcode == opcodes::OP_BRANCH {
            let (taken, target) = cpu.branch_predictor.predict_branch(current_pc);
            if taken {
                if let Some(tgt) = target {
                    next_pc_calc = tgt;
                    pred_taken = true;
                    pred_target = tgt;
                    stop_fetch = true;
                }
            }
        } else if opcode == opcodes::OP_JAL {
            if let Some(tgt) = cpu.branch_predictor.predict_btb(current_pc) {
                next_pc_calc = tgt;
                pred_taken = true;
                pred_target = tgt;
                stop_fetch = true;
            }
        } else if opcode == opcodes::OP_JALR {
            if rd == abi::REG_ZERO && rs1 == abi::REG_RA {
                if let Some(tgt) = cpu.branch_predictor.predict_return() {
                    next_pc_calc = tgt;
                    pred_taken = true;
                    pred_target = tgt;
                }
            } else if let Some(tgt) = cpu.branch_predictor.predict_btb(current_pc) {
                next_pc_calc = tgt;
                pred_taken = true;
                pred_target = tgt;
            }
            stop_fetch = true;
        }

        fetched.push(IfIdEntry {
            pc: current_pc,
            inst,
            pred_taken,
            pred_target,
        });

        current_pc = next_pc_calc;

        if stop_fetch {
            break;
        }
    }

    cpu.pc = current_pc;
    cpu.if_id = IfId { entries: fetched };
    Ok(())
}
