// <FILE>src/random/cls_fast_seeded_random.rs</FILE> - <DESC>Fast seeded random using hash-based RNG</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>RNG performance optimization</WCTX>
// <CLOG>Initial creation - ~25x faster than ChaCha8Rng variant</CLOG>

use crate::math::{fast_random, finite_or, finite_or_f64};
use crate::traits::{Signal, SignalContext, SignalTime};
use serde::{Deserialize, Serialize};

/// Fast seeded random value generator using hash-based RNG.
///
/// ~25x faster than `SeededRandom` by using SplitMix64 mixing instead of ChaCha8Rng.
/// Suitable for animation/visualization where cryptographic quality isn't needed.
///
/// Output is deterministic: same seed + time = same value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FastSeededRandom {
    seed: u64,
    amplitude: f32,
    offset: f32,
}

impl FastSeededRandom {
    pub fn new(seed: u64, amplitude: f32, offset: f32) -> Self {
        Self {
            seed,
            amplitude,
            offset,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed, 1.0, 0.0)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }
}

impl Default for FastSeededRandom {
    fn default() -> Self {
        Self {
            seed: 0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for FastSeededRandom {
    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        let time_ms = (t * 1000.0) as u64;
        let value = fast_random(self.seed, time_ms);
        (offset + value * amplitude).clamp(0.0, 1.0)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        let effective_seed = self.seed.wrapping_add(ctx.seed);
        let time_ms = (t * 1000.0) as u64;
        let combined_input = time_ms.wrapping_add(ctx.frame);
        let value = fast_random(effective_seed, combined_input);
        (offset + value * amplitude).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_seeded_random_determinism() {
        let signal = FastSeededRandom::with_seed(12345);
        let v1 = signal.sample(0.5);
        let v2 = signal.sample(0.5);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_fast_seeded_random_different_times() {
        let signal = FastSeededRandom::with_seed(42);
        let v1 = signal.sample(0.1);
        let v2 = signal.sample(0.2);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_fast_seeded_random_range() {
        let signal = FastSeededRandom::new(42, 0.5, 0.0);
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = signal.sample(t);
            assert!((0.0..=0.5).contains(&v), "Value {} out of range", v);
        }
    }

    #[test]
    fn test_fast_seeded_random_context() {
        let signal = FastSeededRandom::with_seed(42);
        let ctx = SignalContext::new(100, 999);
        let v1 = signal.sample_with_context(0.5, &ctx);
        let v2 = signal.sample_with_context(0.5, &ctx);
        assert_eq!(v1, v2);
    }
}

// <FILE>src/random/cls_fast_seeded_random.rs</FILE> - <DESC>Fast seeded random using hash-based RNG</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
