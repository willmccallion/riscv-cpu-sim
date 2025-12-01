pub struct RegisterFile {
    regs: [u64; 32],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn read(&self, idx: usize) -> u64 {
        if idx == 0 { 0 } else { self.regs[idx] }
    }

    pub fn write(&mut self, idx: usize, val: u64) {
        if idx != 0 {
            self.regs[idx] = val;
        }
    }

    pub fn dump(&self) -> [u64; 32] {
        self.regs
    }
}
