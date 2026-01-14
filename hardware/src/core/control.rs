use super::pipeline::{ExMem, IdEx, IdExEntry, IfId, MemWb};

#[derive(Clone, Copy, Debug, Default)]
pub enum AluOp {
    #[default]
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Rem,
    Remu,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FSqrt,
    FMin,
    FMax,
    FMAdd,
    FMSub,
    FNMAdd,
    FNMSub,
    FCvtWS,
    FCvtLS,
    FCvtSW,
    FCvtSL,
    FCvtSD,
    FCvtDS,
    FSgnJ,
    FSgnJN,
    FSgnJX,
    FEq,
    FLt,
    FLe,
    FClass,
    FMvToX,
    FMvToF,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AtomicOp {
    #[default]
    None,
    Lr,
    Sc,
    Swap,
    Add,
    Xor,
    And,
    Or,
    Min,
    Max,
    Minu,
    Maxu,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MemWidth {
    #[default]
    Nop,
    Byte,
    Half,
    Word,
    Double,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum OpASrc {
    #[default]
    Reg1,
    Pc,
    Zero,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum OpBSrc {
    #[default]
    Imm,
    Reg2,
    Zero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CsrOp {
    #[default]
    None,
    Rw,
    Rs,
    Rc,
    Rwi,
    Rsi,
    Rci,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlSignals {
    pub reg_write: bool,
    pub fp_reg_write: bool,
    pub mem_read: bool,
    pub mem_write: bool,
    pub branch: bool,
    pub jump: bool,
    pub is_rv32: bool,
    pub width: MemWidth,
    pub signed_load: bool,
    pub alu: AluOp,
    pub a_src: OpASrc,
    pub b_src: OpBSrc,
    pub is_system: bool,
    pub csr_addr: u32,
    pub is_mret: bool,
    pub is_sret: bool,
    pub csr_op: CsrOp,
    pub rs1_fp: bool,
    pub rs2_fp: bool,
    pub rs3_fp: bool,
    pub atomic_op: AtomicOp,
}

pub fn need_stall_load_use(id_ex: &IdEx, if_id: &IfId) -> bool {
    for ex_inst in &id_ex.entries {
        if !ex_inst.ctrl.mem_read {
            continue;
        }

        if !ex_inst.ctrl.fp_reg_write && ex_inst.rd == 0 {
            continue;
        }

        for id_inst in &if_id.entries {
            let inst = id_inst.inst;
            let next_rs1 = ((inst >> 15) & 0x1f) as usize;
            let next_rs2 = ((inst >> 20) & 0x1f) as usize;
            let next_rs3 = ((inst >> 27) & 0x1f) as usize;

            if ex_inst.rd == next_rs1 || ex_inst.rd == next_rs2 || ex_inst.rd == next_rs3 {
                return true;
            }
        }
    }
    false
}

pub fn forward_rs(id_entry: &IdExEntry, ex_mem: &ExMem, mem_wb: &MemWb) -> (u64, u64, u64) {
    let mut a = id_entry.rv1;
    let mut b = id_entry.rv2;
    let mut c = id_entry.rv3;

    let check = |dest: usize, dest_fp: bool, src: usize, src_fp: bool| -> bool {
        if dest_fp != src_fp {
            return false;
        }
        if dest != src {
            return false;
        }
        if !dest_fp && dest == 0 {
            return false;
        }
        true
    };

    for wb_entry in mem_wb.entries.iter() {
        if wb_entry.ctrl.reg_write || wb_entry.ctrl.fp_reg_write {
            let wb_val = if wb_entry.ctrl.mem_read {
                wb_entry.load_data
            } else if wb_entry.ctrl.jump {
                wb_entry.pc.wrapping_add(4)
            } else {
                wb_entry.alu
            };

            let dest_fp = wb_entry.ctrl.fp_reg_write;

            if check(wb_entry.rd, dest_fp, id_entry.rs1, id_entry.ctrl.rs1_fp) {
                a = wb_val;
            }
            if check(wb_entry.rd, dest_fp, id_entry.rs2, id_entry.ctrl.rs2_fp) {
                b = wb_val;
            }
            if check(wb_entry.rd, dest_fp, id_entry.rs3, id_entry.ctrl.rs3_fp) {
                c = wb_val;
            }
        }
    }

    for mem_entry in ex_mem.entries.iter() {
        if (mem_entry.ctrl.reg_write || mem_entry.ctrl.fp_reg_write) && !mem_entry.ctrl.mem_read {
            let ex_val = if mem_entry.ctrl.jump {
                mem_entry.pc.wrapping_add(4)
            } else {
                mem_entry.alu
            };

            let dest_fp = mem_entry.ctrl.fp_reg_write;

            if check(mem_entry.rd, dest_fp, id_entry.rs1, id_entry.ctrl.rs1_fp) {
                a = ex_val;
            }
            if check(mem_entry.rd, dest_fp, id_entry.rs2, id_entry.ctrl.rs2_fp) {
                b = ex_val;
            }
            if check(mem_entry.rd, dest_fp, id_entry.rs3, id_entry.ctrl.rs3_fp) {
                c = ex_val;
            }
        }
    }

    (a, b, c)
}
