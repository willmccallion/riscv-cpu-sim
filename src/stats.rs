#[derive(Default, Debug)]
pub struct SimStats {
    pub cycles: u64,
    pub instructions_retired: u64,
    pub branch_predictions: u64,
    pub branch_mispredictions: u64,
    pub icache_hits: u64,
    pub icache_misses: u64,
    pub dcache_hits: u64,
    pub dcache_misses: u64,
}

impl SimStats {
    pub fn print(&self) {
        println!("\n-----------------------------");
        println!("Cycles:               {}", self.cycles);
        println!("Instructions Retired: {}", self.instructions_retired);

        let ipc = if self.cycles > 0 {
            self.instructions_retired as f64 / self.cycles as f64
        } else {
            0.0
        };
        println!("IPC:                  {:.4}", ipc);

        let total_branches = self.branch_predictions;
        if total_branches > 0 {
            let accuracy = 1.0 - (self.branch_mispredictions as f64 / total_branches as f64);
            println!(
                "Branch Prediction:    {:.2}% accuracy ({} / {})",
                accuracy * 100.0,
                total_branches - self.branch_mispredictions,
                total_branches
            );
        } else {
            println!("Branch Prediction:    N/A");
        }

        let total_i = self.icache_hits + self.icache_misses;
        if total_i > 0 {
            let rate = self.icache_hits as f64 / total_i as f64;
            println!(
                "I-Cache:              {:.2}% hit rate ({} / {})",
                rate * 100.0,
                self.icache_hits,
                total_i
            );
        }

        let total_d = self.dcache_hits + self.dcache_misses;
        if total_d > 0 {
            let rate = self.dcache_hits as f64 / total_d as f64;
            println!(
                "D-Cache:              {:.2}% hit rate ({} / {})",
                rate * 100.0,
                self.dcache_hits,
                total_d
            );
        }
        println!("-----------------------------");
    }
}
