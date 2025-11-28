#[derive(Clone, Default)]
struct CacheLine {
    tag: u64,
    valid: bool,
    last_used: u64, // For LRU policy
}

pub struct CacheSim {
    sets: Vec<Vec<CacheLine>>, // [set_index][way_index]
    num_sets: usize,
    ways: usize,
    line_bytes: usize,
    access_counter: u64, // Global time for LRU
}

impl CacheSim {
    pub fn new(size_bytes: usize, line_bytes: usize, ways: usize) -> Self {
        let num_lines = size_bytes / line_bytes;
        let num_sets = num_lines / ways;

        // Initialize sets with empty lines
        let sets = vec![vec![CacheLine::default(); ways]; num_sets];

        Self {
            sets,
            num_sets,
            ways,
            line_bytes,
            access_counter: 0,
        }
    }

    pub fn access(&mut self, addr: u64) -> bool {
        self.access_counter += 1;

        let index = ((addr as usize) / self.line_bytes) % self.num_sets;
        let tag = addr / (self.line_bytes * self.num_sets) as u64;

        // Check for Hit
        for i in 0..self.ways {
            if self.sets[index][i].valid && self.sets[index][i].tag == tag {
                self.sets[index][i].last_used = self.access_counter; // Update LRU
                return true; // Hit
            }
        }

        // Find Invalid line OR Least Recently Used (LRU)
        let mut replace_idx = 0;
        let mut min_lru = u64::MAX;

        for i in 0..self.ways {
            if !self.sets[index][i].valid {
                replace_idx = i;
                break; // Found empty slot, use it
            }
            if self.sets[index][i].last_used < min_lru {
                min_lru = self.sets[index][i].last_used;
                replace_idx = i;
            }
        }

        // Replace
        self.sets[index][replace_idx] = CacheLine {
            tag,
            valid: true,
            last_used: self.access_counter,
        };

        false // Miss
    }
}
