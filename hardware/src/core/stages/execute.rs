use crate::core::Cpu;
use crate::core::control::{AluOp, CsrOp, OpASrc, OpBSrc};
use crate::core::pipeline::{ExMem, ExMemEntry, IfId};
use crate::core::types::Trap;
use crate::isa::{abi, funct3, opcodes, sys_ops};

fn box_f32(f: f32) -> u64 {
    (f.to_bits() as u64) | 0xFFFF_FFFF_0000_0000
}

fn alu(op: AluOp, a: u64, b: u64, c: u64, is32: bool) -> u64 {
    let sh6 = (b & 0x3f) as u32;
    match op {
        AluOp::Add => {
            if is32 {
                (a as i32).wrapping_add(b as i32) as i64 as u64
            } else {
                a.wrapping_add(b)
            }
        }
        AluOp::Sub => {
            if is32 {
                (a as i32).wrapping_sub(b as i32) as i64 as u64
            } else {
                a.wrapping_sub(b)
            }
        }
        AluOp::Sll => {
            if is32 {
                (a as i32).wrapping_shl(b as u32 & 0x1f) as i64 as u64
            } else {
                a.wrapping_shl(sh6)
            }
        }
        AluOp::Srl => {
            if is32 {
                ((a as u32).wrapping_shr(b as u32 & 0x1f)) as i32 as i64 as u64
            } else {
                a.wrapping_shr(sh6)
            }
        }
        AluOp::Sra => {
            if is32 {
                ((a as i32) >> (b as u32 & 0x1f)) as i64 as u64
            } else {
                ((a as i64) >> sh6) as u64
            }
        }
        AluOp::Or => a | b,
        AluOp::And => a & b,
        AluOp::Xor => a ^ b,
        AluOp::Slt => {
            if is32 {
                ((a as i32) < (b as i32)) as u64
            } else {
                ((a as i64) < (b as i64)) as u64
            }
        }
        AluOp::Sltu => {
            if is32 {
                ((a as u32) < (b as u32)) as u64
            } else {
                (a < b) as u64
            }
        }
        AluOp::Mul => {
            if is32 {
                (a as i32).wrapping_mul(b as i32) as i64 as u64
            } else {
                a.wrapping_mul(b)
            }
        }
        AluOp::Mulh => {
            if is32 {
                ((a as i32 as i64 * b as i32 as i64) >> 32) as u64
            } else {
                (((a as i128) * (b as i128)) >> 64) as u64
            }
        }
        AluOp::Mulhsu => {
            if is32 {
                ((a as i32 as i64 * (b as u32) as i64) >> 32) as u64
            } else {
                (((a as i128) * (b as u128 as i128)) >> 64) as u64
            }
        }
        AluOp::Mulhu => {
            if is32 {
                (((a as u32) as u64 * (b as u32) as u64) >> 32) as i64 as u64
            } else {
                (((a as u128) * (b as u128)) >> 64) as u64
            }
        }
        AluOp::Div => {
            if is32 {
                if (b as i32) == 0 {
                    -1i64 as u64
                } else {
                    (a as i32).wrapping_div(b as i32) as i64 as u64
                }
            } else if b == 0 {
                -1i64 as u64
            } else {
                (a as i64).wrapping_div(b as i64) as u64
            }
        }
        AluOp::Divu => {
            if is32 {
                if (b as i32) == 0 {
                    -1i64 as u64
                } else {
                    ((a as u32) / (b as u32)) as i64 as u64
                }
            } else if b == 0 {
                -1i64 as u64
            } else {
                a / b
            }
        }
        AluOp::Rem => {
            if is32 {
                if (b as i32) == 0 {
                    a
                } else {
                    (a as i32).wrapping_rem(b as i32) as i64 as u64
                }
            } else if b == 0 {
                a
            } else {
                (a as i64).wrapping_rem(b as i64) as u64
            }
        }
        AluOp::Remu => {
            if is32 {
                if (b as i32) == 0 {
                    a
                } else {
                    ((a as u32) % (b as u32)) as i64 as u64
                }
            } else if b == 0 {
                a
            } else {
                a % b
            }
        }
        _ => {
            if is32 {
                let fa = f32::from_bits(a as u32);
                let fb = f32::from_bits(b as u32);
                let fc = f32::from_bits(c as u32);
                match op {
                    AluOp::FAdd => box_f32(fa + fb),
                    AluOp::FSub => box_f32(fa - fb),
                    AluOp::FMul => box_f32(fa * fb),
                    AluOp::FDiv => box_f32(fa / fb),
                    AluOp::FSqrt => box_f32(fa.sqrt()),
                    AluOp::FMin => box_f32(fa.min(fb)),
                    AluOp::FMax => box_f32(fa.max(fb)),
                    AluOp::FMAdd => box_f32(fa.mul_add(fb, fc)),
                    AluOp::FMSub => box_f32(fa.mul_add(fb, -fc)),
                    AluOp::FNMAdd => box_f32((-fa).mul_add(fb, -fc)),
                    AluOp::FNMSub => box_f32((-fa).mul_add(fb, fc)),
                    AluOp::FSgnJ => box_f32(f32::from_bits(
                        (fa.to_bits() & !0x8000_0000) | (fb.to_bits() & 0x8000_0000),
                    )),
                    AluOp::FSgnJN => box_f32(f32::from_bits(
                        (fa.to_bits() & !0x8000_0000) | (!fb.to_bits() & 0x8000_0000),
                    )),
                    AluOp::FSgnJX => {
                        box_f32(f32::from_bits(fa.to_bits() ^ (fb.to_bits() & 0x8000_0000)))
                    }
                    AluOp::FEq => (fa == fb) as u64,
                    AluOp::FLt => (fa < fb) as u64,
                    AluOp::FLe => (fa <= fb) as u64,
                    AluOp::FCvtWS => (fa as i32) as i64 as u64,
                    AluOp::FCvtLS => (fa as i64) as u64,
                    AluOp::FCvtSD => box_f32(fa as f32),
                    AluOp::FCvtSW => ((a as i32) as f64).to_bits(),
                    AluOp::FCvtSL => ((a as i64) as f64).to_bits(),
                    AluOp::FCvtDS => (f32::from_bits(a as u32) as f64).to_bits(),
                    AluOp::FMvToF => box_f32(f32::from_bits(a as u32)),
                    AluOp::FMvToX => (a as i32) as u64,
                    _ => 0,
                }
            } else {
                let fa = f64::from_bits(a);
                let fb = f64::from_bits(b);
                let fc = f64::from_bits(c);
                match op {
                    AluOp::FAdd => (fa + fb).to_bits(),
                    AluOp::FSub => (fa - fb).to_bits(),
                    AluOp::FMul => (fa * fb).to_bits(),
                    AluOp::FDiv => (fa / fb).to_bits(),
                    AluOp::FSqrt => fa.sqrt().to_bits(),
                    AluOp::FMin => fa.min(fb).to_bits(),
                    AluOp::FMax => fa.max(fb).to_bits(),
                    AluOp::FMAdd => fa.mul_add(fb, fc).to_bits(),
                    AluOp::FMSub => fa.mul_add(fb, -fc).to_bits(),
                    AluOp::FNMAdd => (-fa).mul_add(fb, -fc).to_bits(),
                    AluOp::FNMSub => (-fa).mul_add(fb, fc).to_bits(),
                    AluOp::FSgnJ => f64::from_bits(
                        (fa.to_bits() & !0x8000_0000_0000_0000)
                            | (fb.to_bits() & 0x8000_0000_0000_0000),
                    )
                    .to_bits(),
                    AluOp::FSgnJN => f64::from_bits(
                        (fa.to_bits() & !0x8000_0000_0000_0000)
                            | (!fb.to_bits() & 0x8000_0000_0000_0000),
                    )
                    .to_bits(),
                    AluOp::FSgnJX => {
                        f64::from_bits(fa.to_bits() ^ (fb.to_bits() & 0x8000_0000_0000_0000))
                            .to_bits()
                    }
                    AluOp::FEq => (fa == fb) as u64,
                    AluOp::FLt => (fa < fb) as u64,
                    AluOp::FLe => (fa <= fb) as u64,
                    AluOp::FCvtWS => (fa as i32) as i64 as u64,
                    AluOp::FCvtLS => (fa as i64) as u64,
                    AluOp::FCvtSD => box_f32(fa as f32),
                    AluOp::FCvtSW => ((a as i32) as f64).to_bits(),
                    AluOp::FCvtSL => ((a as i64) as f64).to_bits(),
                    AluOp::FMvToF => a,
                    AluOp::FMvToX => a,
                    _ => 0,
                }
            }
        }
    }
}

pub fn execute_stage(cpu: &mut Cpu) -> Result<(), String> {
    let mut ex_results = Vec::new();
    let mut flush_remaining = false;

    let entries = cpu.id_ex.entries.clone();

    for id in entries {
        if flush_remaining {
            break;
        }

        if let Some(trap) = id.trap.clone() {
            ex_results.push(ExMemEntry {
                pc: id.pc,
                inst: id.inst,
                rd: id.rd,
                alu: 0,
                store_data: 0,
                ctrl: id.ctrl,
                trap: Some(trap),
            });
            continue;
        }

        if cpu.trace {
            eprintln!("EX  pc={:#x}", id.pc);
        }

        let (fwd_a, fwd_b, fwd_c) =
            crate::core::control::forward_rs(&id, &cpu.ex_mem, &cpu.wb_latch);
        let store_data = fwd_b;

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
        let op_c = fwd_c;

        if id.ctrl.is_system {
            if id.ctrl.is_mret {
                cpu.do_mret();
                flush_remaining = true;
                cpu.if_id = IfId::default();
                continue;
            }
            if id.ctrl.is_sret {
                cpu.do_sret();
                flush_remaining = true;
                cpu.if_id = IfId::default();
                continue;
            }

            if id.inst == sys_ops::SFENCE_VMA {
                if cpu.trace {
                    eprintln!("EX  SFENCE.VMA - Flushing TLBs");
                }
                cpu.mmu.dtlb.flush();
                cpu.mmu.itlb.flush();
                ex_results.push(ExMemEntry {
                    pc: id.pc,
                    inst: id.inst,
                    rd: id.rd,
                    alu: 0,
                    store_data,
                    ctrl: id.ctrl,
                    trap: None,
                });
                continue;
            }

            if id.inst == sys_ops::ECALL {
                // Helper to resolve register values including current bundle
                let get_val = |reg: usize, cpu: &Cpu, current_results: &[ExMemEntry]| -> u64 {
                    if reg == 0 {
                        return 0;
                    }
                    for entry in current_results.iter().rev() {
                        if entry.ctrl.reg_write && entry.rd == reg {
                            return entry.alu;
                        }
                    }

                    for entry in cpu.ex_mem.entries.iter().rev() {
                        if entry.ctrl.reg_write && entry.rd == reg {
                            return if entry.ctrl.jump {
                                entry.pc.wrapping_add(4)
                            } else {
                                entry.alu
                            };
                        }
                    }

                    for entry in cpu.mem_wb.entries.iter().rev() {
                        if entry.ctrl.reg_write && entry.rd == reg {
                            return if entry.ctrl.mem_read {
                                entry.load_data
                            } else if entry.ctrl.jump {
                                entry.pc.wrapping_add(4)
                            } else {
                                entry.alu
                            };
                        }
                    }
                    cpu.regs.read(reg)
                };

                // Only intercept SYS_EXIT if we are in Direct Mode (no kernel).
                if cpu.direct_mode {
                    let val_a7 = get_val(abi::REG_A7, cpu, &ex_results);
                    let val_a0 = get_val(abi::REG_A0, cpu, &ex_results);

                    if val_a7 == sys_ops::SYS_EXIT {
                        cpu.exit_code = Some(val_a0);
                        return Ok(());
                    } else if val_a0 == sys_ops::SYS_EXIT {
                        // Handle legacy/alternative convention if needed
                        let val_a1 = get_val(abi::REG_A1, cpu, &ex_results);
                        cpu.exit_code = Some(val_a1);
                        return Ok(());
                    }
                }

                let trap = match cpu.privilege {
                    0 => Trap::EnvironmentCallFromUMode,
                    1 => Trap::EnvironmentCallFromSMode,
                    3 => Trap::EnvironmentCallFromMMode,
                    _ => Trap::EnvironmentCallFromMMode,
                };

                cpu.trap(trap, id.pc);
                flush_remaining = true;
                continue;
            }

            if id.ctrl.csr_op != CsrOp::None {
                let old = cpu.csr_read(id.ctrl.csr_addr);
                let src = match id.ctrl.csr_op {
                    CsrOp::Rwi | CsrOp::Rsi | CsrOp::Rci => (id.rs1 as u64) & 0x1f,
                    _ => fwd_a,
                };
                let new = match id.ctrl.csr_op {
                    CsrOp::Rw | CsrOp::Rwi => src,
                    CsrOp::Rs | CsrOp::Rsi => old | src,
                    CsrOp::Rc | CsrOp::Rci => old & !src,
                    CsrOp::None => old,
                };
                cpu.csr_write(id.ctrl.csr_addr, new);

                cpu.if_id = IfId::default();
                cpu.pc = id.pc.wrapping_add(4);
                flush_remaining = true;

                ex_results.push(ExMemEntry {
                    pc: id.pc,
                    inst: id.inst,
                    rd: id.rd,
                    alu: old,
                    store_data,
                    ctrl: id.ctrl,
                    trap: None,
                });
                continue;
            }
        }

        let alu_out = if (id.ctrl.alu as i32 >= AluOp::FCvtSW as i32
            && id.ctrl.alu as i32 <= AluOp::FCvtSL as i32)
            || id.ctrl.alu as i32 == AluOp::FMvToF as i32
        {
            match id.ctrl.alu {
                AluOp::FCvtSW => {
                    if id.ctrl.is_rv32 {
                        box_f32((op_a as i32) as f32)
                    } else {
                        ((op_a as i32) as f64).to_bits()
                    }
                }
                AluOp::FCvtSL => {
                    if id.ctrl.is_rv32 {
                        box_f32((op_a as i64) as f32)
                    } else {
                        ((op_a as i64) as f64).to_bits()
                    }
                }
                AluOp::FCvtSD => {
                    let val_d = f64::from_bits(op_a);
                    let val_s = val_d as f32;
                    box_f32(val_s)
                }
                AluOp::FCvtDS => {
                    let val_s = f32::from_bits(op_a as u32);
                    let val_d = val_s as f64;
                    val_d.to_bits()
                }
                AluOp::FMvToF => {
                    if id.ctrl.is_rv32 {
                        box_f32(f32::from_bits(op_a as u32))
                    } else {
                        op_a
                    }
                }
                _ => 0,
            }
        } else {
            alu(id.ctrl.alu, op_a, op_b, op_c, id.ctrl.is_rv32)
        };

        if id.ctrl.branch {
            let taken = match (id.inst >> 12) & 0x7 {
                funct3::BEQ => op_a == op_b,
                funct3::BNE => op_a != op_b,
                funct3::BLT => (op_a as i64) < (op_b as i64),
                funct3::BGE => (op_a as i64) >= (op_b as i64),
                funct3::BLTU => op_a < op_b,
                funct3::BGEU => op_a >= op_b,
                _ => false,
            };
            let actual_target = id.pc.wrapping_add(id.imm as u64);
            let fallthrough = id.pc.wrapping_add(4);

            let predicted_target = if id.pred_taken {
                id.pred_target
            } else {
                fallthrough
            };
            let actual_next_pc = if taken { actual_target } else { fallthrough };

            let mispredicted = predicted_target != actual_next_pc;

            cpu.branch_predictor.update_branch(
                id.pc,
                taken,
                if taken { Some(actual_target) } else { None },
            );

            if mispredicted {
                cpu.stats.branch_mispredictions += 1;
                cpu.stats.stalls_control += 2;

                cpu.pc = actual_next_pc;
                cpu.if_id = IfId::default();
                flush_remaining = true;
            } else {
                cpu.stats.branch_predictions += 1;
            }
        }

        if id.ctrl.jump {
            let is_jalr = (id.inst & 0x7f) == opcodes::OP_JALR;
            let is_call = (id.inst & 0x7f) == opcodes::OP_JAL && id.rd == abi::REG_RA;
            let is_ret = is_jalr && id.rd == abi::REG_ZERO && id.rs1 == abi::REG_RA;

            let actual_target = if is_jalr {
                (fwd_a.wrapping_add(id.imm as u64)) & !1
            } else {
                id.pc.wrapping_add(id.imm as u64)
            };

            let predicted_target = if id.pred_taken {
                id.pred_target
            } else {
                id.pc.wrapping_add(4)
            };

            if actual_target != predicted_target {
                cpu.stats.branch_mispredictions += 1;
                cpu.stats.stalls_control += 2;
                cpu.pc = actual_target;
                cpu.if_id = IfId::default();
                flush_remaining = true;
            } else {
                cpu.stats.branch_predictions += 1;
            }

            if is_call {
                cpu.branch_predictor
                    .on_call(id.pc, id.pc.wrapping_add(4), actual_target);
            } else if is_ret {
                cpu.branch_predictor.on_return();
            }
        }

        ex_results.push(ExMemEntry {
            pc: id.pc,
            inst: id.inst,
            rd: id.rd,
            alu: alu_out,
            store_data,
            ctrl: id.ctrl,
            trap: None,
        });
    }

    cpu.ex_mem = ExMem {
        entries: ex_results,
    };
    Ok(())
}
