mod uart;
mod virtio_disk;

pub use uart::Uart;
pub use virtio_disk::{VIRTUAL_DISK_SIZE_ADDRESS, VirtualDisk};

use crate::memory::Memory;

/// UART @ 0x1000_0000 … 0x1000_0007
const UART_BASE: u64 = 0x1000_0000;
const UART_SIZE: u64 = 8;

pub struct Bus {
    pub mem: Memory,
    pub uart: Uart,
    pub disk: VirtualDisk,
}

impl Bus {
    pub fn new(mem: Memory, uart: Uart, disk: VirtualDisk) -> Self {
        Self { mem, uart, disk }
    }

    #[inline]
    fn is_uart(addr: u64) -> bool {
        (UART_BASE..(UART_BASE + UART_SIZE)).contains(&addr)
    }

    pub fn read_u8(&mut self, paddr: u64) -> u8 {
        if Self::is_uart(paddr) {
            self.uart.read_u8(paddr)
        } else if self.disk.is_in_range(paddr) {
            self.disk.read_u8(paddr)
        } else {
            self.mem.read_u8(paddr)
        }
    }
    pub fn read_u16(&mut self, paddr: u64) -> u16 {
        if self.disk.is_in_range(paddr) {
            self.disk.read_u16(paddr)
        } else {
            self.mem.read_u16(paddr)
        }
    }
    pub fn read_u32(&mut self, paddr: u64) -> u32 {
        if self.disk.is_in_range(paddr) {
            let v = self.disk.read_u32(paddr);
            if (VIRTUAL_DISK_SIZE_ADDRESS..VIRTUAL_DISK_SIZE_ADDRESS + 8).contains(&paddr) {
                eprintln!("BUS: u32 read disk-size @ {:#x} -> {:#x}", paddr, v);
            }
            v
        } else {
            self.mem.read_u32(paddr)
        }
    }

    pub fn read_u64(&mut self, paddr: u64) -> u64 {
        if self.disk.is_in_range(paddr) {
            self.disk.read_u64(paddr)
        } else {
            self.mem.read_u64(paddr)
        }
    }

    pub fn write_u8(&mut self, paddr: u64, v: u8) {
        if Self::is_uart(paddr) {
            self.uart.write_u8(paddr, v);
        } else if self.disk.is_in_range(paddr) { /* ignore writes to disk regs */
        } else {
            self.mem.write_u8(paddr, v);
        }
    }
    pub fn write_u16(&mut self, paddr: u64, v: u16) {
        if self.disk.is_in_range(paddr) { /* ignore */
        } else {
            self.mem.write_u16(paddr, v);
        }
    }
    pub fn write_u32(&mut self, paddr: u64, v: u32) {
        if self.disk.is_in_range(paddr) { /* ignore */
        } else {
            self.mem.write_u32(paddr, v);
        }
    }
    pub fn write_u64(&mut self, paddr: u64, v: u64) {
        if self.disk.is_in_range(paddr) { /* ignore */
        } else {
            self.mem.write_u64(paddr, v);
        }
    }
}
