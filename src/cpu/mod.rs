pub mod branch_predictor;
pub mod cache;
pub mod control;
pub mod pipeline;
pub mod stages;

use self::branch_predictor::BranchPredictor;
use self::cache::CacheSim;
use crate::devices::Bus;
use crate::register_file::RegisterFile;
use crate::stats::SimStats;
use pipeline::{EXMEM, IDEx, IFID, MEMWB};
use stages::{decode_stage, execute_stage, fetch_stage, mem_stage, wb_stage};

pub const MEMORY_LATENCY: u64 = 10;

/// Debug-only println
#[macro_export]
macro_rules! dbg_println {
    () => { #[cfg(debug_assertions)] { eprintln!(); } };
    ($($arg:tt)*) => { #[cfg(debug_assertions)] { eprintln!($($arg)*); } };
}

#[derive(Default)]
struct Csrs {
    mstatus: u64,
    sstatus: u64,
    mepc: u64,
    sepc: u64,
    mtvec: u64,
    satp: u64,
}

pub struct Cpu {
    pub regs: RegisterFile,
    pub pc: u64,
    pub trace: bool,
    pub bus: Bus,
    pub exit_code: Option<u64>,

    csrs: Csrs,
    privilege: u8,

    if_id: IFID,
    id_ex: IDEx,
    ex_mem: EXMEM,
    pub mem_wb: MEMWB,
    pub wb_latch: MEMWB,

    pub stats: SimStats,
    pub branch_predictor: BranchPredictor,
    pub i_cache: CacheSim,
    pub d_cache: CacheSim,

    pub stall_cycles: u64,
}

impl Cpu {
    pub fn new(reset_pc: u64, trace: bool, bus: Bus) -> Self {
        Self {
            regs: RegisterFile::new(),
            pc: reset_pc,
            trace,
            bus,
            exit_code: None,
            csrs: Csrs::default(),
            privilege: 3, // Machine
            if_id: IFID::default(),
            id_ex: IDEx::default(),
            ex_mem: EXMEM::default(),
            mem_wb: MEMWB::default(),
            wb_latch: MEMWB::default(),

            stats: SimStats::default(),
            branch_predictor: BranchPredictor::new(),
            // 16KB I-Cache, 64B lines, 4 ways
            i_cache: CacheSim::new(16 * 1024, 64, 4),
            // 16KB D-Cache, 64B lines, 4 ways
            d_cache: CacheSim::new(16 * 1024, 64, 4),

            stall_cycles: 0,
        }
    }

    pub fn tick(&mut self) -> Result<(), String> {
        if self.stall_cycles > 0 {
            self.stall_cycles -= 1;
            self.stats.cycles += 1;
            return Ok(());
        }

        self.stats.cycles += 1;

        wb_stage(self);

        self.wb_latch = self.mem_wb;

        mem_stage(self)?;
        execute_stage(self)?;

        let is_load_use_hazard =
            crate::cpu::control::need_stall_load_use(&self.id_ex, self.if_id.inst);

        if is_load_use_hazard {
            self.id_ex = IDEx::default();
        } else {
            decode_stage(self)?;
            fetch_stage(self)?;
        }
        // keep x0 hardwired
        self.regs.write(0, 0);
        Ok(())
    }

    pub fn print_stats(&self) {
        self.stats.print();
    }

    pub fn take_exit(&mut self) -> Option<u64> {
        self.exit_code.take()
    }

    pub fn dump_state(&self) {
        println!("PC = {:#018x}", self.pc);
        let r = self.regs.dump();
        for i in (0..32).step_by(2) {
            println!(
                "x{:<2} = {:#018x}    x{:<2} = {:#018x}",
                i,
                r[i],
                i + 1,
                r[i + 1]
            );
        }
    }

    #[inline]
    pub fn read_inst(&mut self, pc: u64) -> u32 {
        self.bus.read_u32(pc)
    }

    #[inline]
    pub fn load_u8(&mut self, a: u64) -> u8 {
        self.bus.read_u8(a)
    }

    #[inline]
    pub fn load_u16(&mut self, a: u64) -> u16 {
        self.bus.read_u16(a)
    }

    #[inline]
    pub fn load_u32(&mut self, a: u64) -> u32 {
        self.bus.read_u32(a)
    }

    #[inline]
    pub fn load_u64(&mut self, a: u64) -> u64 {
        self.bus.read_u64(a)
    }

    #[inline]
    pub fn store_u8(&mut self, a: u64, v: u8) {
        self.bus.write_u8(a, v)
    }

    #[inline]
    pub fn store_u16(&mut self, a: u64, v: u16) {
        self.bus.write_u16(a, v)
    }

    #[inline]
    pub fn store_u32(&mut self, a: u64, v: u32) {
        self.bus.write_u32(a, v)
    }

    #[inline]
    pub fn store_u64(&mut self, a: u64, v: u64) {
        self.bus.write_u64(a, v)
    }

    pub(crate) fn csr_read(&self, addr: u32) -> u64 {
        match addr {
            0x300 => self.csrs.mstatus,
            0x100 => self.csrs.sstatus,
            0x341 => self.csrs.mepc,
            0x141 => self.csrs.sepc,
            0x305 => self.csrs.mtvec,
            0x180 => self.csrs.satp,
            _ => 0,
        }
    }

    pub(crate) fn csr_write(&mut self, addr: u32, val: u64) {
        match addr {
            0x300 => self.csrs.mstatus = val,
            0x100 => self.csrs.sstatus = val,
            0x341 => {
                // MEPC: bit0 is WARL=0 (alignment)
                self.csrs.mepc = val & !1;
                if self.trace {
                    eprintln!("CSR write: mepc <- {:#x} (masked to even)", self.csrs.mepc);
                }
            }
            0x141 => {
                // SEPC: bit0 is WARL=0 (alignment)
                self.csrs.sepc = val & !1;
                if self.trace {
                    eprintln!("CSR write: sepc <- {:#x} (masked to even)", self.csrs.sepc);
                }
            }
            0x305 => self.csrs.mtvec = val,
            0x180 => self.csrs.satp = val,
            _ => {}
        }
    }

    pub(crate) fn do_mret(&mut self) {
        let target = self.csrs.mepc & !1; // enforce alignment
        assert!(target != 0, "MRET with mepc=0 (no trap/entry set)");
        self.pc = target;
        self.privilege = 1; // drop to S-mode in this tiny model
        self.if_id = IFID::default();
        self.id_ex = IDEx::default();
        if self.trace {
            eprintln!("MRET -> pc={:#x}", self.pc);
        }
    }

    pub(crate) fn do_sret(&mut self) {
        let target = self.csrs.sepc & !1; // enforce alignment
        assert!(target != 0, "SRET with sepc=0 (no entry set)");
        self.pc = target;
        self.privilege = 0;
        self.if_id = IFID::default();
        self.id_ex = IDEx::default();
        if self.trace {
            eprintln!("SRET -> pc={:#x}", self.pc);
        }
    }
}
