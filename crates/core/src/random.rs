//! Caller-controlled pseudo-randomness for deterministic game simulation.

/// A source of uniformly distributed integers for reward selection.
///
/// Implementations must return a value in `0..upper_exclusive`. Keeping this
/// interface in core lets tests inject exact boundary rolls while front-ends
/// decide how production games are seeded.
pub trait RandomSource {
    /// Returns one value below `upper_exclusive`.
    fn below(&mut self, upper_exclusive: u32) -> u32;
}

/// A small deterministic generator suitable for gameplay randomness.
///
/// This is SplitMix64. It is not cryptographically secure, but identical seeds
/// produce identical reward sequences across the TUI and headless tools.
#[derive(Debug, Clone)]
pub struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    /// Creates a generator from an explicit seed.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

impl RandomSource for SeededRandom {
    fn below(&mut self, upper_exclusive: u32) -> u32 {
        assert!(upper_exclusive > 0, "random upper bound must be positive");
        // Multiply-high maps a uniform u64 into the requested range without
        // modulo bias.
        ((self.next_u64() as u128 * upper_exclusive as u128) >> 64) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_sequences_are_reproducible_and_bounded() {
        let mut first = SeededRandom::new(42);
        let mut second = SeededRandom::new(42);
        let a: Vec<u32> = (0..20).map(|_| first.below(100)).collect();
        let b: Vec<u32> = (0..20).map(|_| second.below(100)).collect();
        assert_eq!(a, b);
        assert!(a.iter().all(|&value| value < 100));
    }
}
