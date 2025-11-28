use crate::cpu::Cpu;
use crate::cpu::control::{AluOp, CsrOp, OpASrc, OpBSrc};
use crate::cpu::pipeline::EXMEM;
use crate::isa::{funct3, opcodes};

fn alu(op: AluOp, a: u64, b: u64, is32: bool) -> u64 {
    let sh6 = (b & 0x3f) as u32;
    if is32 {
        let a32 = a as i32;
        let b32 = b as i32;
        let s5 = (b & 0x1f) as u32;
        let r = match op {
            AluOp::Add => a32.wrapping_add(b32) as i64,
            AluOp::Sub => a32.wrapping_sub(b32) as i64,
            AluOp::Sll => a32.wrapping_shl(s5) as i64,
            AluOp::Srl => ((a32 as u32).wrapping_shr(s5)) as i32 as i64,
            AluOp::Sra => (a32 >> s5) as i64,
            AluOp::Or => (a32 | b32) as i64,
            AluOp::And => (a32 & b32) as i64,
            AluOp::Xor => (a32 ^ b32) as i64,
            AluOp::Slt => (a32 < b32) as i64,
            AluOp::Sltu => ((a as u32) < (b as u32)) as i64,
            AluOp::Mul => a32.wrapping_mul(b32) as i64,
            AluOp::Mulh => ((a32 as i64 * b32 as i64) >> 32) as i64,
            AluOp::Mulhsu => ((a32 as i64 * (b as u32) as i64) >> 32) as i64,
            AluOp::Mulhu => (((a as u32) as u64 * (b as u32) as u64) >> 32) as i64,
            AluOp::Div => {
                if b32 == 0 {
                    -1
                } else {
                    a32.wrapping_div(b32) as i64
                }
            }
            AluOp::Rem => {
                if b32 == 0 {
                    a32 as i64
                } else {
                    a32.wrapping_rem(b32) as i64
                }
            }
            AluOp::Divu => {
                if b32 == 0 {
                    -1
                } else {
                    ((a as u32) / (b as u32)) as i32 as i64
                }
            }
            AluOp::Remu => {
                if b32 == 0 {
                    a32 as i64
                } else {
                    ((a as u32) % (b as u32)) as i32 as i64
                }
            }
        };
        r as u64
    } else {
        match op {
            AluOp::Add => a.wrapping_add(b),
            AluOp::Sub => a.wrapping_sub(b),
            AluOp::Sll => a.wrapping_shl(sh6),
            AluOp::Srl => a.wrapping_shr(sh6),
            AluOp::Sra => ((a as i64) >> sh6) as u64,
            AluOp::Or => a | b,
            AluOp::And => a & b,
            AluOp::Xor => a ^ b,
            AluOp::Slt => ((a as i64) < (b as i64)) as u64,
            AluOp::Sltu => (a < b) as u64,
            AluOp::Mul => a.wrapping_mul(b),
            AluOp::Mulh => (((a as i128) * (b as i128)) >> 64) as u64,
            AluOp::Mulhsu => (((a as i128) * (b as u128 as i128)) >> 64) as u64,
            AluOp::Mulhu => (((a as u128) * (b as u128)) >> 64) as u64,
            AluOp::Div => {
                if b == 0 {
                    -1i64 as u64
                } else {
                    (a as i64).wrapping_div(b as i64) as u64
                }
            }
            AluOp::Divu => {
                if b == 0 {
                    -1i64 as u64
                } else {
                    a / b
                }
            }
            AluOp::Rem => {
                if b == 0 {
                    a
                } else {
                    (a as i64).wrapping_rem(b as i64) as u64
                }
            }
            AluOp::Remu => {
                if b == 0 {
                    a
                } else {
                    a % b
                }
            }
        }
    }
}

pub fn execute_stage(cpu: &mut Cpu) -> Result<(), String> {
    let id = cpu.id_ex;
    if cpu.trace {
        eprintln!(
            "EX  pc={:#x} inst={:#010x} (rs1={}, rs2={}, rd={})",
            id.pc, id.inst, id.rs1, id.rs2, id.rd
        );
        eprintln!("    ID values: rv1={:#x}, rv2={:#x}", id.rv1, id.rv2);
    }

    let (fwd_a, fwd_b) = crate::cpu::control::forward_rs(&cpu.id_ex, &cpu.ex_mem, &cpu.wb_latch);

    if cpu.trace && (id.rs1 != 0 || id.rs2 != 0) {
        let (fwd_a_dbg, fwd_b_dbg) =
            crate::cpu::control::forward_rs(&cpu.id_ex, &cpu.ex_mem, &cpu.wb_latch);

        if fwd_a_dbg != id.rv1 {
            eprintln!(
                "    FORWARD: rs1 ({}) value {:#x} -> {:#x}",
                id.rs1, id.rv1, fwd_a_dbg
            );
        }
        if fwd_b_dbg != id.rv2 {
            eprintln!(
                "    FORWARD: rs2 ({}) value {:#x} -> {:#x}",
                id.rs2, id.rv2, fwd_b_dbg
            );
        }
    }

    let store_data_val = fwd_b;

    let op_a = match id.ctrl.a_src {
        OpASrc::Reg1 => fwd_a,
        OpASrc::Pc => id.pc,
        OpASrc::Zero => 0,
    };

    let op_b = match id.ctrl.b_src {
        OpBSrc::Reg2 => fwd_b,
        OpBSrc::Imm => id.imm as u64,
        OpBSrc::Zero => 0,
    };

    if id.ctrl.is_system {
        if id.ctrl.is_mret {
            cpu.do_mret();
        } else if id.ctrl.is_sret {
            cpu.do_sret();
        }

        if id.ctrl.csr_op != CsrOp::None
            && !id.ctrl.is_mret
            && !id.ctrl.is_sret
            && id.inst != 0x0000_0073
        {
            let addr = id.ctrl.csr_addr;
            let old = cpu.csr_read(addr);
            let src = match id.ctrl.csr_op {
                CsrOp::RWI | CsrOp::RSI | CsrOp::RCI => (id.rs1 as u64) & 0x1f,
                _ => fwd_a,
            };
            let new = match id.ctrl.csr_op {
                CsrOp::RW | CsrOp::RWI => src,
                CsrOp::RS | CsrOp::RSI => old | src,
                CsrOp::RC | CsrOp::RCI => old & !src,
                CsrOp::None => old,
            };
            cpu.csr_write(addr, new);

            cpu.ex_mem = EXMEM {
                pc: id.pc,
                inst: id.inst,
                rd: id.rd,
                alu: old,
                store_data: store_data_val,
                ctrl: id.ctrl,
            };
            return Ok(());
        }
    }

    let alu_out = alu(id.ctrl.alu, op_a, op_b, id.ctrl.is_rv32);

    if id.ctrl.branch {
        let taken = match (id.inst >> 12) & 0x7 {
            funct3::BEQ => op_a == op_b,
            funct3::BNE => op_a != op_b,
            funct3::BLT => (op_a as i64) < (op_b as i64),
            funct3::BGE => (op_a as i64) >= (op_b as i64),
            funct3::BLTU => op_a < op_b,
            funct3::BGEU => op_a >= (op_b as u64),
            _ => false,
        };

        // Branch Prediction Logic
        let prediction = cpu.branch_predictor.predict(id.pc);
        cpu.stats.branch_predictions += 1;
        if prediction != taken {
            cpu.stats.branch_mispredictions += 1;
        }
        cpu.branch_predictor.update(id.pc, taken);

        if taken {
            cpu.pc = id.pc.wrapping_add(id.imm as u64);
            cpu.if_id = Default::default();
            cpu.id_ex = Default::default();
        }
    }

    if id.ctrl.jump {
        let is_jalr = (id.inst & 0x7f) == opcodes::OP_JALR;
        let target = if is_jalr {
            (fwd_a.wrapping_add(id.imm as u64)) & !1
        } else {
            id.pc.wrapping_add(id.imm as u64)
        };
        cpu.pc = target;
        cpu.if_id = Default::default();
        cpu.id_ex = Default::default();
    }

    cpu.ex_mem = EXMEM {
        pc: id.pc,
        inst: id.inst,
        rd: id.rd,
        alu: alu_out,
        store_data: store_data_val,
        ctrl: id.ctrl,
    };
    Ok(())
}
