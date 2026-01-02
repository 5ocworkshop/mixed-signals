// <FILE>src/random/cls_per_character_noise.rs</FILE> - <DESC>Per-character deterministic noise using character index</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, rng_from_time, u64_to_bipolar, scale_bipolar</CLOG>

use crate::core::{bipolar_range, scale_bipolar, u64_to_bipolar};
use crate::math::derive_seed;
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Per-character noise generator producing deterministic variance based on character index.
///
/// Uses character index as a seed modifier to produce consistent random values per character.
/// Same character index always produces the same value, regardless of time or frame.
/// Useful for typewriter speed variance, scramble timing per character, etc.
///
/// **Requires:** `char_index` field in `SignalContext`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PerCharacterNoise {
    /// Base seed for reproducible randomness
    base_seed: u64,
    /// Output amplitude (scales the 0..1 range)
    amplitude: f32,
    /// Center value
    offset: f32,
}

impl PerCharacterNoise {
    pub fn new(base_seed: u64, amplitude: f32, offset: f32) -> Self {
        Self {
            base_seed,
            amplitude,
            offset,
        }
    }

    pub fn with_seed(base_seed: u64) -> Self {
        Self::new(base_seed, 1.0, 0.0)
    }

    pub fn base_seed(&self) -> u64 {
        self.base_seed
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }
}

impl Default for PerCharacterNoise {
    fn default() -> Self {
        Self {
            base_seed: 0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for PerCharacterNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        // In absence of char_index, use time as a fallback
        let pseudo_index = (crate::math::finite_or_f64(t, 0.0) * 100.0) as u64;
        let seed_bytes = derive_seed(self.base_seed, pseudo_index);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let bipolar = u64_to_bipolar(rng.next_u64());
        scale_bipolar(bipolar, self.amplitude, self.offset)
    }

    fn sample_with_context(&self, _t: SignalTime, ctx: &SignalContext) -> f32 {
        // Use char_index from context if available, otherwise fallback to frame
        let char_index = ctx.char_index.unwrap_or(ctx.frame as usize) as u64;
        let effective_seed = self.base_seed.wrapping_add(ctx.seed);
        let seed_bytes = derive_seed(effective_seed, char_index);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let bipolar = u64_to_bipolar(rng.next_u64());
        scale_bipolar(bipolar, self.amplitude, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_character_noise_determinism() {
        let noise = PerCharacterNoise::with_seed(12345);
        let ctx = SignalContext::new(5, 0); // frame=5 acts as char_index placeholder
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        assert_eq!(v1, v2, "Same char_index should produce same value");
    }

    #[test]
    fn test_per_character_noise_different_indices() {
        let noise = PerCharacterNoise::with_seed(42);
        let ctx1 = SignalContext::new(0, 0); // char_index=0
        let ctx2 = SignalContext::new(1, 0); // char_index=1

        let v1 = noise.sample_with_context(0.0, &ctx1);
        let v2 = noise.sample_with_context(0.0, &ctx2);

        assert_ne!(
            v1, v2,
            "Different char_index should produce different values"
        );
    }

    #[test]
    fn test_per_character_noise_time_independent() {
        let noise = PerCharacterNoise::with_seed(42);
        let ctx = SignalContext::new(5, 0);

        // Same char_index, different times should produce same value
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(100.0, &ctx);

        assert_eq!(v1, v2, "Time should not affect per-character noise");
    }

    #[test]
    fn test_per_character_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = PerCharacterNoise::new(42, 0.5, 0.0);

        for i in 0..100 {
            let ctx = SignalContext::new(i, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!(
                (-0.5..=0.5).contains(&v),
                "Value {} out of expected range [-0.5, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_per_character_noise_offset() {
        // amplitude=0.5, offset=0.25 -> range [-0.25, 0.75]
        let noise = PerCharacterNoise::new(42, 0.5, 0.25);

        for i in 0..100 {
            let ctx = SignalContext::new(i, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!(
                (-0.25..=0.75).contains(&v),
                "Value {} out of expected range [-0.25, 0.75] with offset",
                v
            );
        }
    }

    #[test]
    fn test_per_character_noise_output_range() {
        let noise = PerCharacterNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_per_character_noise_different_seeds() {
        let noise1 = PerCharacterNoise::with_seed(1);
        let noise2 = PerCharacterNoise::with_seed(2);
        let ctx = SignalContext::new(5, 0);

        let v1 = noise1.sample_with_context(0.0, &ctx);
        let v2 = noise2.sample_with_context(0.0, &ctx);

        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_per_character_noise_distribution() {
        let noise = PerCharacterNoise::with_seed(42);

        // Verify that values are distributed (not all the same)
        let mut values = Vec::new();
        for i in 0..100 {
            let ctx = SignalContext::new(i, 0);
            values.push(noise.sample_with_context(0.0, &ctx));
        }

        // Calculate variance
        let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
        let variance: f32 =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;

        // Variance should be non-zero (values are distributed)
        assert!(
            variance > 0.01,
            "Variance {} should indicate distributed values",
            variance
        );
    }

    #[test]
    fn test_per_character_noise_finite() {
        let noise = PerCharacterNoise::with_seed(42);

        for i in 0..1000 {
            let ctx = SignalContext::new(i, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }
}

// <FILE>src/random/cls_per_character_noise.rs</FILE> - <DESC>Per-character deterministic noise using character index</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
