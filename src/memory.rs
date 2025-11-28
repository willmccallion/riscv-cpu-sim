pub const MEMORY_SIZE: usize = 128 * 1024 * 1024; // 128 MiB
pub const BASE_ADDRESS: u64 = 0x8000_0000;

pub struct Memory {
    bytes: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            bytes: vec![0; MEMORY_SIZE],
        }
    }

    pub fn load(&mut self, data: &[u8], offset: usize) {
        assert!(offset + data.len() <= self.bytes.len(), "Memory::load OOB");
        self.bytes[offset..offset + data.len()].copy_from_slice(data);
    }

    #[inline]
    fn idx(&self, paddr: u64, size: usize) -> usize {
        let i = paddr.checked_sub(BASE_ADDRESS).unwrap_or(paddr) as usize;
        assert!(
            i + size <= self.bytes.len(),
            "mem OOB addr={:#x} size={}",
            paddr,
            size
        );
        i
    }

    pub fn read_u8(&self, paddr: u64) -> u8 {
        self.bytes[self.idx(paddr, 1)]
    }

    pub fn read_u16(&self, paddr: u64) -> u16 {
        let i = self.idx(paddr, 2);
        u16::from_le_bytes(self.bytes[i..i + 2].try_into().unwrap())
    }

    pub fn read_u32(&self, paddr: u64) -> u32 {
        let i = self.idx(paddr, 4);
        u32::from_le_bytes(self.bytes[i..i + 4].try_into().unwrap())
    }

    pub fn read_u64(&self, paddr: u64) -> u64 {
        let i = self.idx(paddr, 8);
        u64::from_le_bytes(self.bytes[i..i + 8].try_into().unwrap())
    }

    pub fn write_u8(&mut self, paddr: u64, v: u8) {
        let i = self.idx(paddr, 1);
        self.bytes[i] = v;
    }

    pub fn write_u16(&mut self, paddr: u64, v: u16) {
        let i = self.idx(paddr, 2);
        self.bytes[i..i + 2].copy_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32(&mut self, paddr: u64, v: u32) {
        let i = self.idx(paddr, 4);
        self.bytes[i..i + 4].copy_from_slice(&v.to_le_bytes());
    }

    pub fn write_u64(&mut self, paddr: u64, v: u64) {
        let i = self.idx(paddr, 8);
        self.bytes[i..i + 8].copy_from_slice(&v.to_le_bytes());
    }
}
