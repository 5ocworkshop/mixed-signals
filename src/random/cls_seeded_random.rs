// <FILE>mixed-signals/src/random/cls_seeded_random.rs</FILE> - <DESC>Seeded random value generator for deterministic randomness</DESC>
// <VERS>VERSION: 1.1.0</VERS>
// <WCTX>mixed-signals extraction: RNG standardization</WCTX>
// <CLOG>Replaced custom hash with ChaCha8Rng for reproducible randomness</CLOG>

use crate::math::{derive_seed, finite_or, finite_or_f64};
use crate::traits::{Signal, SignalContext, SignalTime};
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Seeded random value generator producing deterministic uniform random values.
///
/// Uses ChaCha8Rng for reproducible randomness.
/// Each call to `sample()` with different time values produces different random values,
/// but the sequence is reproducible given the same seed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeededRandom {
    /// Seed for reproducible randomness
    seed: u64,
    /// Output amplitude (scales the 0..1 range)
    amplitude: f32,
    /// Center value
    offset: f32,
}

impl SeededRandom {
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

impl Default for SeededRandom {
    fn default() -> Self {
        Self {
            seed: 0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for SeededRandom {
    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        // Convert time to integer for seeding (quantize to milliseconds)
        let time_ms = (t * 1000.0) as u64;
        let seed_bytes = derive_seed(self.seed, time_ms);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);

        // Generate random value in 0..u64::MAX range
        let random_value = rng.next_u64();
        // Normalize to 0.0 to 1.0 range
        let normalized = random_value as f64 / u64::MAX as f64;
        (offset + normalized as f32 * amplitude).clamp(0.0, 1.0)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        // Incorporate context seed and frame for additional randomness
        let effective_seed = self.seed.wrapping_add(ctx.seed);
        let time_ms = (t * 1000.0) as u64;
        let combined_input = time_ms.wrapping_add(ctx.frame);
        let seed_bytes = derive_seed(effective_seed, combined_input);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);

        let random_value = rng.next_u64();
        let normalized = random_value as f64 / u64::MAX as f64;
        (offset + normalized as f32 * amplitude).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeded_random_determinism() {
        let signal = SeededRandom::with_seed(12345);
        let v1 = signal.sample(0.5);
        let v2 = signal.sample(0.5);
        assert_eq!(v1, v2, "Same time should produce same value");
    }

    #[test]
    fn test_seeded_random_different_times() {
        let signal = SeededRandom::with_seed(42);
        let v1 = signal.sample(0.1);
        let v2 = signal.sample(0.2);
        assert_ne!(v1, v2, "Different times should produce different values");
    }

    #[test]
    fn test_seeded_random_different_seeds() {
        let signal1 = SeededRandom::with_seed(1);
        let signal2 = SeededRandom::with_seed(2);
        let v1 = signal1.sample(0.5);
        let v2 = signal2.sample(0.5);
        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_seeded_random_range() {
        let signal = SeededRandom::new(42, 0.5, 0.0);
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = signal.sample(t);
            assert!(
                (0.0..=0.5).contains(&v),
                "Value {} out of expected range [0.0, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_seeded_random_offset() {
        let signal = SeededRandom::new(42, 0.5, 0.25);
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = signal.sample(t);
            assert!(
                (0.25..=0.75).contains(&v),
                "Value {} out of expected range [0.25, 0.75] with offset",
                v
            );
        }
    }

    #[test]
    fn test_seeded_random_context() {
        let signal = SeededRandom::with_seed(42);
        let ctx = SignalContext::new(100, 999);
        let v1 = signal.sample_with_context(0.5, &ctx);
        let v2 = signal.sample_with_context(0.5, &ctx);
        assert_eq!(v1, v2, "Same context should produce same value");
    }

    #[test]
    fn test_seeded_random_finite() {
        let signal = SeededRandom::with_seed(42);
        for i in 0..1000 {
            let t = i as f64 * 0.1;
            let v = signal.sample(t);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }
}

// <FILE>mixed-signals/src/random/cls_seeded_random.rs</FILE> - <DESC>Seeded random value generator for deterministic randomness</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
