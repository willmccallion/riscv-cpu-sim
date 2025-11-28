/// A simple 2-bit Saturating Counter Branch Predictor
/// Table size: 1024 entries
pub struct BranchPredictor {
    table: Vec<u8>, // 0..=3
}

impl BranchPredictor {
    pub fn new() -> Self {
        Self {
            table: vec![2; 1024], // Initialize to Weakly Taken (2)
        }
    }

    fn index(&self, pc: u64) -> usize {
        // Use bits [11:2] of PC for index
        ((pc >> 2) & 0x3FF) as usize
    }

    pub fn predict(&self, pc: u64) -> bool {
        let state = self.table[self.index(pc)];
        state >= 2
    }

    pub fn update(&mut self, pc: u64, taken: bool) {
        let idx = self.index(pc);
        let state = self.table[idx];
        if taken && state < 3 {
            self.table[idx] += 1;
        } else if state > 0 {
            self.table[idx] -= 1;
        }
    }
}
