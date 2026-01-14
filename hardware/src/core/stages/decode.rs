use crate::core::Cpu;
use crate::core::control::{AluOp, AtomicOp, ControlSignals, CsrOp, MemWidth, OpASrc, OpBSrc};
use crate::core::pipeline::{IdEx, IdExEntry};
use crate::core::types::Trap;
use crate::isa::instruction::InstructionBits;
use crate::isa::{decoder, funct3, funct5, funct7, opcodes, sys_ops};

pub fn decode_stage(cpu: &mut Cpu) -> Result<(), String> {
    let mut decoded = Vec::new();
    let mut consumed_count = 0;
    let mut bundle_writes: Vec<(usize, bool)> = Vec::new();

    for if_entry in &cpu.if_id.entries {
        let inst = if_entry.inst;
        if inst == 0x0000_0013 || inst == 0 {
            consumed_count += 1;
            continue;
        }

        let d = decoder::decode(inst);

        let decode_logic = |d: &crate::isa::instruction::Decoded| -> Result<ControlSignals, Trap> {
            let mut c = ControlSignals {
                a_src: OpASrc::Reg1,
                b_src: OpBSrc::Imm,
                alu: AluOp::Add,
                ..Default::default()
            };

            match d.opcode {
                opcodes::OP_LUI => {
                    c.reg_write = true;
                    c.a_src = OpASrc::Zero;
                }
                opcodes::OP_AUIPC => {
                    c.reg_write = true;
                    c.a_src = OpASrc::Pc;
                }
                opcodes::OP_JAL => {
                    c.reg_write = true;
                    c.jump = true;
                    c.a_src = OpASrc::Pc;
                }
                opcodes::OP_JALR => {
                    c.reg_write = true;
                    c.jump = true;
                    c.a_src = OpASrc::Reg1;
                }
                opcodes::OP_BRANCH => {
                    c.branch = true;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Reg2;
                }
                opcodes::OP_LOAD => {
                    c.reg_write = true;
                    c.mem_read = true;
                    c.a_src = OpASrc::Reg1;
                    match d.funct3 {
                        funct3::LB => {
                            c.width = MemWidth::Byte;
                            c.signed_load = true;
                        }
                        funct3::LH => {
                            c.width = MemWidth::Half;
                            c.signed_load = true;
                        }
                        funct3::LW => {
                            c.width = MemWidth::Word;
                            c.signed_load = true;
                        }
                        funct3::LD => {
                            c.width = MemWidth::Double;
                            c.signed_load = true;
                        }
                        funct3::LBU => {
                            c.width = MemWidth::Byte;
                            c.signed_load = false;
                        }
                        funct3::LHU => {
                            c.width = MemWidth::Half;
                            c.signed_load = false;
                        }
                        funct3::LWU => {
                            c.width = MemWidth::Word;
                            c.signed_load = false;
                        }
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    }
                }
                opcodes::OP_STORE => {
                    c.mem_write = true;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Imm;
                    match d.funct3 {
                        funct3::SB => c.width = MemWidth::Byte,
                        funct3::SH => c.width = MemWidth::Half,
                        funct3::SW => c.width = MemWidth::Word,
                        funct3::SD => c.width = MemWidth::Double,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    }
                }
                opcodes::OP_IMM | opcodes::OP_IMM_32 => {
                    c.reg_write = true;
                    c.is_rv32 = d.opcode == opcodes::OP_IMM_32;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Imm;
                    c.alu = match d.funct3 {
                        funct3::ADD_SUB => AluOp::Add,
                        funct3::SLL => AluOp::Sll,
                        funct3::SLT => AluOp::Slt,
                        funct3::SLTU => AluOp::Sltu,
                        funct3::XOR => AluOp::Xor,
                        funct3::SRL_SRA => {
                            if (d.funct7 & 0x20) != 0 {
                                AluOp::Sra
                            } else {
                                AluOp::Srl
                            }
                        }
                        funct3::OR => AluOp::Or,
                        funct3::AND => AluOp::And,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    };
                }
                opcodes::OP_REG | opcodes::OP_REG_32 => {
                    c.reg_write = true;
                    c.is_rv32 = d.opcode == opcodes::OP_REG_32;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Reg2;
                    c.alu = match (d.funct3, d.funct7) {
                        (funct3::ADD_SUB, funct7::DEFAULT) => AluOp::Add,
                        (funct3::ADD_SUB, funct7::SUB) => AluOp::Sub,
                        (funct3::SLL, funct7::DEFAULT) => AluOp::Sll,
                        (funct3::SLT, funct7::DEFAULT) => AluOp::Slt,
                        (funct3::SLTU, funct7::DEFAULT) => AluOp::Sltu,
                        (funct3::XOR, funct7::DEFAULT) => AluOp::Xor,
                        (funct3::SRL_SRA, funct7::DEFAULT) => AluOp::Srl,
                        (funct3::SRL_SRA, funct7::SRA) => AluOp::Sra,
                        (funct3::OR, funct7::DEFAULT) => AluOp::Or,
                        (funct3::AND, funct7::DEFAULT) => AluOp::And,
                        (funct3::ADD_SUB, funct7::M_EXTENSION) => AluOp::Mul,
                        (funct3::SLL, funct7::M_EXTENSION) => AluOp::Mulh,
                        (funct3::SLT, funct7::M_EXTENSION) => AluOp::Mulhsu,
                        (funct3::SLTU, funct7::M_EXTENSION) => AluOp::Mulhu,
                        (funct3::XOR, funct7::M_EXTENSION) => AluOp::Div,
                        (funct3::SRL_SRA, funct7::M_EXTENSION) => AluOp::Divu,
                        (funct3::OR, funct7::M_EXTENSION) => AluOp::Rem,
                        (funct3::AND, funct7::M_EXTENSION) => AluOp::Remu,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    };
                }
                opcodes::OP_SYSTEM => {
                    c.is_system = true;
                    match d.raw {
                        sys_ops::ECALL => {}
                        sys_ops::EBREAK => return Err(Trap::Breakpoint(if_entry.pc)),
                        sys_ops::MRET => c.is_mret = true,
                        sys_ops::SRET => c.is_sret = true,
                        sys_ops::WFI => {}
                        sys_ops::SFENCE_VMA => {}
                        _ => {
                            c.csr_addr = inst.csr();
                            c.a_src = OpASrc::Reg1;
                            c.b_src = OpBSrc::Zero;
                            c.reg_write = d.rd != 0;
                            c.csr_op = match d.funct3 {
                                sys_ops::CSRRW => CsrOp::Rw,
                                sys_ops::CSRRS => CsrOp::Rs,
                                sys_ops::CSRRC => CsrOp::Rc,
                                sys_ops::CSRRWI => CsrOp::Rwi,
                                sys_ops::CSRRSI => CsrOp::Rsi,
                                sys_ops::CSRRCI => CsrOp::Rci,
                                _ => return Err(Trap::IllegalInstruction(inst)),
                            };
                        }
                    }
                }
                opcodes::OP_AMO => {
                    c.reg_write = true;
                    c.mem_read = true;
                    c.mem_write = true;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Zero;
                    match d.funct3 {
                        funct3::LW => c.width = MemWidth::Word,
                        funct3::LD => c.width = MemWidth::Double,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    }
                    let f5 = d.funct7 >> 2;
                    c.atomic_op = match f5 {
                        funct5::LR => {
                            c.mem_write = false;
                            AtomicOp::Lr
                        }
                        funct5::SC => {
                            c.mem_read = false;
                            AtomicOp::Sc
                        }
                        funct5::AMOSWAP => AtomicOp::Swap,
                        funct5::AMOADD => AtomicOp::Add,
                        funct5::AMOXOR => AtomicOp::Xor,
                        funct5::AMOAND => AtomicOp::And,
                        funct5::AMOOR => AtomicOp::Or,
                        funct5::AMOMIN => AtomicOp::Min,
                        funct5::AMOMAX => AtomicOp::Max,
                        funct5::AMOMINU => AtomicOp::Minu,
                        funct5::AMOMAXU => AtomicOp::Maxu,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    };
                }
                opcodes::OP_LOAD_FP => {
                    c.fp_reg_write = true;
                    c.mem_read = true;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Imm;
                    match d.funct3 {
                        0x2 => c.width = MemWidth::Word,
                        0x3 => c.width = MemWidth::Double,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    }
                }
                opcodes::OP_STORE_FP => {
                    c.mem_write = true;
                    c.rs2_fp = true;
                    c.a_src = OpASrc::Reg1;
                    c.b_src = OpBSrc::Imm;
                    match d.funct3 {
                        0x2 => c.width = MemWidth::Word,
                        0x3 => c.width = MemWidth::Double,
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    }
                }
                opcodes::OP_FP => {
                    c.fp_reg_write = true;
                    c.rs1_fp = true;
                    c.rs2_fp = true;
                    c.b_src = OpBSrc::Reg2;
                    let fmt = d.funct7 & 0x3;
                    c.is_rv32 = fmt == 0;
                    let op = d.funct7 >> 2;
                    match op {
                        funct7::FADD => c.alu = AluOp::FAdd,
                        funct7::FSUB => c.alu = AluOp::FSub,
                        funct7::FMUL => c.alu = AluOp::FMul,
                        funct7::FDIV => c.alu = AluOp::FDiv,
                        funct7::FSQRT => {
                            c.rs2_fp = false;
                            c.alu = AluOp::FSqrt;
                        }
                        funct7::FSGNJ => match d.funct3 {
                            funct3::FSGNJ => c.alu = AluOp::FSgnJ,
                            funct3::FSGNJN => c.alu = AluOp::FSgnJN,
                            funct3::FSGNJX => c.alu = AluOp::FSgnJX,
                            _ => return Err(Trap::IllegalInstruction(inst)),
                        },
                        funct7::FMIN_MAX => match d.funct3 {
                            funct3::FMIN => c.alu = AluOp::FMin,
                            funct3::FMAX => c.alu = AluOp::FMax,
                            _ => return Err(Trap::IllegalInstruction(inst)),
                        },
                        funct7::FCMP => {
                            c.fp_reg_write = false;
                            c.reg_write = true;
                            match d.funct3 {
                                funct3::FEQ => c.alu = AluOp::FEq,
                                funct3::FLT => c.alu = AluOp::FLt,
                                funct3::FLE => c.alu = AluOp::FLe,
                                _ => return Err(Trap::IllegalInstruction(inst)),
                            }
                        }
                        funct7::FCLASS_MV_X_F => match d.funct3 {
                            funct3::FMV_X_W => {
                                c.fp_reg_write = false;
                                c.reg_write = true;
                                c.rs1_fp = true;
                                c.rs2_fp = false;
                                c.alu = AluOp::FMvToX;
                            }
                            funct3::FCLASS => {
                                c.fp_reg_write = false;
                                c.reg_write = true;
                                c.rs1_fp = true;
                                c.rs2_fp = false;
                                c.alu = AluOp::FClass;
                            }
                            _ => return Err(Trap::IllegalInstruction(inst)),
                        },
                        funct7::FCVT_W_F => {
                            c.fp_reg_write = false;
                            c.reg_write = true;
                            c.rs1_fp = true;
                            c.rs2_fp = false;
                            c.alu = if d.rs2 == 1 {
                                AluOp::FCvtLS
                            } else {
                                AluOp::FCvtWS
                            };
                        }
                        funct7::FCVT_F_W => {
                            c.fp_reg_write = true;
                            c.reg_write = false;
                            c.rs1_fp = false;
                            c.rs2_fp = false;
                            c.a_src = OpASrc::Reg1;
                            c.alu = if d.rs2 == 1 {
                                AluOp::FCvtSL
                            } else {
                                AluOp::FCvtSW
                            };
                        }
                        funct7::FMV_F_X => {
                            c.fp_reg_write = true;
                            c.reg_write = false;
                            c.rs1_fp = false;
                            c.rs2_fp = false;
                            c.a_src = OpASrc::Reg1;
                            c.alu = AluOp::FMvToF;
                        }
                        funct7::FCVT_DS => {
                            c.rs2_fp = false;
                            if d.rs2 == 1 {
                                c.alu = AluOp::FCvtSD;
                            } else {
                                c.alu = AluOp::FCvtDS;
                            }
                        }
                        _ => return Err(Trap::IllegalInstruction(inst)),
                    }
                }
                opcodes::OP_FMADD | opcodes::OP_FMSUB | opcodes::OP_FNMADD | opcodes::OP_FNMSUB => {
                    c.fp_reg_write = true;
                    c.rs1_fp = true;
                    c.rs2_fp = true;
                    c.rs3_fp = true;
                    c.b_src = OpBSrc::Reg2;
                    c.is_rv32 = (d.funct7 & 3) == 0;
                    c.alu = match d.opcode {
                        opcodes::OP_FMADD => AluOp::FMAdd,
                        opcodes::OP_FMSUB => AluOp::FMSub,
                        opcodes::OP_FNMADD => AluOp::FNMAdd,
                        opcodes::OP_FNMSUB => AluOp::FNMSub,
                        _ => AluOp::Add,
                    };
                }
                _ => return Err(Trap::IllegalInstruction(inst)),
            }
            Ok(c)
        };

        let (ctrl, trap) = match decode_logic(&d) {
            Ok(c) => (c, None),
            Err(t) => (ControlSignals::default(), Some(t)),
        };

        let mut hazard = false;

        if d.rs1 != 0 || ctrl.rs1_fp {
            let is_fp = ctrl.rs1_fp;
            if bundle_writes.contains(&(d.rs1, is_fp)) {
                hazard = true;
            }
        }
        if d.rs2 != 0 || ctrl.rs2_fp {
            let is_fp = ctrl.rs2_fp;
            if bundle_writes.contains(&(d.rs2, is_fp)) {
                hazard = true;
            }
        }

        let rs3_idx = inst.rs3();
        if ctrl.rs3_fp {
            if bundle_writes.contains(&(rs3_idx, true)) {
                hazard = true;
            }
        }

        if hazard {
            break;
        }

        if ctrl.reg_write && d.rd != 0 {
            bundle_writes.push((d.rd, false));
        }
        if ctrl.fp_reg_write {
            bundle_writes.push((d.rd, true));
        }

        let rv1 = if ctrl.rs1_fp {
            cpu.regs.read_f(d.rs1)
        } else {
            cpu.regs.read(d.rs1)
        };
        let rv2 = if ctrl.rs2_fp {
            cpu.regs.read_f(d.rs2)
        } else {
            cpu.regs.read(d.rs2)
        };
        let rv3 = if ctrl.rs3_fp {
            cpu.regs.read_f(rs3_idx)
        } else {
            0
        };

        if cpu.trace {
            eprintln!(
                "ID  pc={:#x} inst={:#08x} rs1=x{} v={:#x} rs2=x{} v={:#x} rd=x{} imm={:#x}",
                if_entry.pc, inst, d.rs1, rv1, d.rs2, rv2, d.rd, d.imm
            );
        }

        decoded.push(IdExEntry {
            pc: if_entry.pc,
            inst,
            rs1: d.rs1,
            rs2: d.rs2,
            rs3: rs3_idx,
            rd: d.rd,
            imm: d.imm,
            rv1,
            rv2,
            rv3,
            ctrl,
            trap,
            pred_taken: if_entry.pred_taken,
            pred_target: if_entry.pred_target,
        });

        consumed_count += 1;
    }

    cpu.id_ex = IdEx { entries: decoded };

    if consumed_count < cpu.if_id.entries.len() {
        cpu.if_id.entries.drain(0..consumed_count);
    } else {
        cpu.if_id.entries.clear();
    }

    Ok(())
}
