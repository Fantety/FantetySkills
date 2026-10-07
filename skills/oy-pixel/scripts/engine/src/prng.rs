/// SplitMix64: tiny, deterministic, well-distributed; used to seed `rand`.
pub struct SplitMix64 {
    pub(crate) state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform float in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform integer in `[a, b]` (inclusive). Returns `a` when `b <= a`.
    /// The span is computed in `i128`: `b - a + 1` overflows `i64` for
    /// near-full-range inputs, and a wrapped span of 0 would make the
    /// remainder below divide by zero and abort the process.
    pub fn next_range(&mut self, a: i64, b: i64) -> i64 {
        if b <= a {
            return a;
        }
        let span = (b as i128) - (a as i128) + 1;
        let offset = (self.next_u64() as i128) % span;
        ((a as i128) + offset) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::SplitMix64;

    #[test]
    fn deterministic_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn floats_and_ranges_stay_in_bounds() {
        let mut rng = SplitMix64::new(7);
        for _ in 0..1000 {
            let f = rng.next_f64();
            assert!((0.0..1.0).contains(&f));
            let v = rng.next_range(-3, 9);
            assert!((-3..=9).contains(&v));
        }
        assert_eq!(SplitMix64::new(1).next_range(5, 5), 5);
        assert_eq!(SplitMix64::new(1).next_range(9, 2), 9);
    }

    #[test]
    fn near_full_range_bounds_stay_in_bounds() {
        // b - a + 1 overflows i64 here; the span math must not wrap to a
        // zero divisor.
        let mut rng = SplitMix64::new(11);
        for _ in 0..10_000 {
            let v = rng.next_range(i64::MIN, i64::MAX);
            assert!((i64::MIN..=i64::MAX).contains(&v));
        }
    }
}
