use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Deterministic random number generator
/// Wraps ChaCha8Rng to ensure reproducible simulations
#[derive(Clone, Serialize, Deserialize)]
pub struct DeterministicRng {
    #[serde(skip, default = "default_rng")]
    rng: ChaCha8Rng,
    seed: u64,
}

fn default_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(0)
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
        }
    }

    pub fn from_entropy() -> Self {
        let seed = rand::random();
        Self::new(seed)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Generate random u32
    pub fn gen_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    /// Generate random u64
    pub fn gen_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Generate random value in range [0, max)
    pub fn gen_range(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        self.gen_u32() % max
    }

    /// Generate random value in range [min, max)
    pub fn gen_range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.gen_range((max - min) as u32) as i32)
    }

    /// Generate random boolean with given probability (0.0 - 1.0)
    pub fn gen_bool(&mut self, probability: f64) -> bool {
        (self.gen_u32() as f64 / u32::MAX as f64) < probability
    }

    /// Limited random - biased toward lower values (from original bugs.c)
    pub fn limited_random(&mut self, interval: u32) -> u32 {
        let mut result = 0;
        let mut i = interval;
        while i > 0 {
            result += self.gen_range(2);
            i /= 2;
        }
        result
    }
}

impl Default for DeterministicRng {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = DeterministicRng::new(12345);
        let mut rng2 = DeterministicRng::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.gen_u32(), rng2.gen_u32());
        }
    }

    #[test]
    fn test_gen_range() {
        let mut rng = DeterministicRng::new(42);
        for _ in 0..100 {
            let val = rng.gen_range(10);
            assert!(val < 10);
        }
    }
}
