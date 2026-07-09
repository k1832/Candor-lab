//! Deterministic pseudo-randomness: a seeded xorshift64 generator.
//!
//! P19 determinism contract: the ONLY source of variation is the caller's seed.
//! There is no `Date`, no environment read, no OS RNG — same seed, byte-identical
//! draw sequence, hence (with a deterministic toolchain filter) a byte-identical
//! corpus. See `README.md#determinism`.

/// A seeded xorshift64 stream (Marsaglia's 13/7/17 triple).
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seed the stream. The state is forced non-zero (xorshift's fixed point),
    /// so every `u64` seed — including 0 — yields a valid stream.
    pub fn new(seed: u64) -> Rng {
        // `^` binds tighter than `|`, so this is `(seed ^ K) | 1`: odd, non-zero.
        Rng {
            state: (seed ^ 0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    /// Advance and return the next 64-bit word.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// A value in the inclusive range `[lo, hi]` (requires `lo <= hi`).
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        debug_assert!(lo <= hi);
        let span = (hi - lo + 1) as u64;
        lo + (self.next_u64() % span) as i64
    }

    /// An index into `[0, n)` (requires `n > 0`).
    pub fn index(&mut self, n: usize) -> usize {
        debug_assert!(n > 0);
        (self.next_u64() % n as u64) as usize
    }

    /// A fair coin.
    pub fn boolean(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}
