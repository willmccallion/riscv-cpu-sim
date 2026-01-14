use crate::system::devices::Device;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SysCon {
    base_addr: u64,
    exit_signal: Arc<AtomicU64>,
}

impl SysCon {
    pub fn new(base_addr: u64, exit_signal: Arc<AtomicU64>) -> Self {
        Self {
            base_addr,
            exit_signal,
        }
    }
}

impl Device for SysCon {
    fn name(&self) -> &str {
        "SysCon"
    }

    fn address_range(&self) -> (u64, u64) {
        (self.base_addr, 0x1000)
    }

    fn read_u8(&mut self, _offset: u64) -> u8 {
        0
    }
    fn read_u16(&mut self, _offset: u64) -> u16 {
        0
    }
    fn read_u32(&mut self, _offset: u64) -> u32 {
        0
    }
    fn read_u64(&mut self, _offset: u64) -> u64 {
        0
    }

    fn write_u8(&mut self, _offset: u64, _val: u8) {}
    fn write_u16(&mut self, _offset: u64, _val: u16) {}

    fn write_u32(&mut self, offset: u64, val: u32) {
        // Register 0: System Control / Status
        if offset == 0 {
            match val {
                0x5555 => {
                    println!("[SysCon] Poweroff signal received.");
                    self.exit_signal.store(0, Ordering::Relaxed)
                }
                0x7777 => {
                    println!("[SysCon] Reset signal received (Simulated as Exit).");
                    self.exit_signal.store(0, Ordering::Relaxed)
                }
                0x3333 => {
                    println!("[SysCon] Failure signal received.");
                    self.exit_signal.store(1, Ordering::Relaxed)
                }
                _ => {}
            }
        }
    }

    fn write_u64(&mut self, offset: u64, val: u64) {
        self.write_u32(offset, val as u32);
    }
}
