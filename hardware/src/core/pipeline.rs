use crate::core::control::ControlSignals;
use crate::core::types::Trap;

#[derive(Clone, Copy, Default, Debug)]
pub struct IfIdEntry {
    pub pc: u64,
    pub inst: u32,
    pub pred_taken: bool,
    pub pred_target: u64,
}

#[derive(Clone, Default, Debug)]
pub struct IdExEntry {
    pub pc: u64,
    pub inst: u32,
    pub rs1: usize,
    pub rs2: usize,
    pub rs3: usize,
    pub rd: usize,
    pub imm: i64,
    pub rv1: u64,
    pub rv2: u64,
    pub rv3: u64,
    pub ctrl: ControlSignals,
    pub trap: Option<Trap>,
    pub pred_taken: bool,
    pub pred_target: u64,
}

#[derive(Clone, Default, Debug)]
pub struct ExMemEntry {
    pub pc: u64,
    pub inst: u32,
    pub rd: usize,
    pub alu: u64,
    pub store_data: u64,
    pub ctrl: ControlSignals,
    pub trap: Option<Trap>,
}

#[derive(Clone, Default, Debug)]
pub struct MemWbEntry {
    pub pc: u64,
    pub inst: u32,
    pub rd: usize,
    pub alu: u64,
    pub load_data: u64,
    pub ctrl: ControlSignals,
    pub trap: Option<Trap>,
}

#[derive(Clone, Debug)]
pub struct IfId {
    pub entries: Vec<IfIdEntry>,
}

impl Default for IfId {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct IdEx {
    pub entries: Vec<IdExEntry>,
}

impl IdEx {
    pub fn bubble() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct ExMem {
    pub entries: Vec<ExMemEntry>,
}

#[derive(Clone, Default, Debug)]
pub struct MemWb {
    pub entries: Vec<MemWbEntry>,
}
