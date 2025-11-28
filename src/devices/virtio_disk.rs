pub const VIRTUAL_DISK_ADDRESS: u64 = 0x9000_0000;
pub const VIRTUAL_DISK_SIZE_ADDRESS: u64 = 0x9000_0FF8;

#[derive(Default)]
pub struct VirtualDisk {
    data: Vec<u8>,
}

impl VirtualDisk {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn load(&mut self, bytes: Vec<u8>) {
        self.data = bytes;
    }

    #[inline]
    fn is_size_reg(addr: u64) -> bool {
        let base = VIRTUAL_DISK_SIZE_ADDRESS;
        (base..base + 8).contains(&addr)
    }

    pub fn is_in_range(&self, addr: u64) -> bool {
        Self::is_size_reg(addr)
            || (addr >= VIRTUAL_DISK_ADDRESS
                && (addr - VIRTUAL_DISK_ADDRESS) < self.data.len() as u64)
    }

    #[inline]
    fn size_le(&self) -> [u8; 8] {
        (self.data.len() as u64).to_le_bytes()
    }

    pub fn read_u8(&self, addr: u64) -> u8 {
        if Self::is_size_reg(addr) {
            let o = (addr - VIRTUAL_DISK_SIZE_ADDRESS) as usize;
            self.size_le()[o]
        } else {
            let o = (addr - VIRTUAL_DISK_ADDRESS) as usize;
            self.data[o]
        }
    }
    pub fn read_u16(&self, addr: u64) -> u16 {
        if Self::is_size_reg(addr) {
            let o = (addr - VIRTUAL_DISK_SIZE_ADDRESS) as usize;
            u16::from_le_bytes([self.size_le()[o], self.size_le()[o + 1]])
        } else {
            let o = (addr - VIRTUAL_DISK_ADDRESS) as usize;
            u16::from_le_bytes(self.data[o..o + 2].try_into().unwrap())
        }
    }
    pub fn read_u32(&self, addr: u64) -> u32 {
        if Self::is_size_reg(addr) {
            let o = (addr - VIRTUAL_DISK_SIZE_ADDRESS) as usize;
            u32::from_le_bytes(self.size_le()[o..o + 4].try_into().unwrap())
        } else {
            let o = (addr - VIRTUAL_DISK_ADDRESS) as usize;
            u32::from_le_bytes(self.data[o..o + 4].try_into().unwrap())
        }
    }
    pub fn read_u64(&self, addr: u64) -> u64 {
        if (VIRTUAL_DISK_SIZE_ADDRESS..VIRTUAL_DISK_SIZE_ADDRESS + 8).contains(&addr) {
            u64::from_le_bytes(self.size_le())
        } else {
            let o = (addr - VIRTUAL_DISK_ADDRESS) as usize;
            u64::from_le_bytes(self.data[o..o + 8].try_into().unwrap())
        }
    }
}
