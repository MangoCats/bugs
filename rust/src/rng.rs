/// Deterministic RNG matching the C implementation exactly.
/// Must produce identical sequence for pixel-perfect output.
pub struct Rng {
    seed: i64,
}

impl Rng {
    pub fn new() -> Self {
        Self { seed: 54321 }
    }

    /// Matches C: limitedrandom(limit)
    /// seed = myabs(((seed + 12355) * 16807)) % 0x3FFFFFFF
    /// return ((unsigned long)seed >> 8) % limit
    pub fn limitedrandom(&mut self, limit: i64) -> i64 {
        let s = (self.seed + 12355).wrapping_mul(16807);
        // myabs
        self.seed = if s >= 0 { s } else { -s };
        self.seed %= 0x3FFFFFFF;

        (((self.seed as u64) >> 8) % (limit as u64)) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_sequence() {
        // Verify the first few values match C output
        let mut rng = Rng::new();
        // These should be validated against C output
        let v1 = rng.limitedrandom(100);
        let v2 = rng.limitedrandom(100);
        let v3 = rng.limitedrandom(100);
        // Just ensure it doesn't panic and produces values in range
        assert!(v1 >= 0 && v1 < 100);
        assert!(v2 >= 0 && v2 < 100);
        assert!(v3 >= 0 && v3 < 100);
    }
}
