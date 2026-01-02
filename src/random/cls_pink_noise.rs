// <FILE>src/random/cls_pink_noise.rs</FILE> - <DESC>1/f noise (pink noise) using stateless multi-octave summation</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, u64_to_bipolar; keep ChaCha8 for quality</CLOG>

use crate::core::{bipolar_range, u64_to_bipolar};
use crate::math::{derive_seed, finite_or, finite_or_f64};
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Pink noise (1/f noise) generator with natural-looking temporal characteristics.
///
/// Pink noise has more low-frequency content than white noise, producing more
/// natural-looking variance. Used in music synthesis, visual effects, and organic animations.
/// Uses stateless ChaCha8Rng-based multi-octave summation for determinism.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PinkNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Output amplitude
    amplitude: f32,
    /// Center value
    offset: f32,
}

impl PinkNoise {
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

impl Default for PinkNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for PinkNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        // Convert time to frame equivalent (assume 60fps)
        let frame = (t * 60.0) as u64;

        // Sum multiple octaves with 1/f amplitude relationship
        let num_octaves = 5;
        let mut sum = 0.0;

        for octave in 0..num_octaves {
            let octave_seed = self.seed.wrapping_add(octave as u64 * 1000);
            let octave_frame = frame >> octave; // Lower frequencies for higher octaves

            let seed_bytes = derive_seed(octave_seed, octave_frame);
            let mut rng = ChaCha8Rng::from_seed(seed_bytes);
            let random_value = rng.next_u64();
            let bipolar = u64_to_bipolar(random_value);

            // 1/f amplitude (each octave has half the amplitude)
            let octave_amplitude = 1.0 / (octave as f32 + 1.0);
            sum += bipolar as f32 * octave_amplitude;
        }

        // Normalize by sum of 1/f series (1 + 1/2 + 1/3 + 1/4 + 1/5 ≈ 2.283)
        let normalizer: f32 = (0..num_octaves).map(|o| 1.0 / (o as f32 + 1.0)).sum();
        offset + (sum / normalizer) * amplitude
    }

    fn sample_with_context(&self, _t: SignalTime, ctx: &SignalContext) -> f32 {
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        // Use context frame directly for temporal consistency
        let frame = ctx.frame;
        let effective_seed = self.seed.wrapping_add(ctx.seed);

        // Sum multiple octaves with 1/f amplitude relationship
        let num_octaves = 5;
        let mut sum = 0.0;

        for octave in 0..num_octaves {
            let octave_seed = effective_seed.wrapping_add(octave as u64 * 1000);
            let octave_frame = frame >> octave; // Lower frequencies for higher octaves

            let seed_bytes = derive_seed(octave_seed, octave_frame);
            let mut rng = ChaCha8Rng::from_seed(seed_bytes);
            let random_value = rng.next_u64();
            let bipolar = u64_to_bipolar(random_value);

            // 1/f amplitude
            let octave_amplitude = 1.0 / (octave as f32 + 1.0);
            sum += bipolar as f32 * octave_amplitude;
        }

        // Normalize
        let normalizer: f32 = (0..num_octaves).map(|o| 1.0 / (o as f32 + 1.0)).sum();
        offset + (sum / normalizer) * amplitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pink_noise_determinism() {
        let noise = PinkNoise::with_seed(12345);
        let ctx = SignalContext::new(100, 0);
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        assert_eq!(v1, v2, "Same frame should produce same value (stateless)");
    }

    #[test]
    fn test_pink_noise_stateless() {
        let noise = PinkNoise::with_seed(42);
        let ctx = SignalContext::new(50, 0);

        // Multiple calls with same context should yield identical output
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        let v3 = noise.sample_with_context(0.0, &ctx);

        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
    }

    #[test]
    fn test_pink_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = PinkNoise::new(42, 0.5, 0.0);

        for frame in 0..100 {
            let ctx = SignalContext::new(frame, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            // Values should be within bipolar amplitude range
            assert!(
                (-0.5..=0.5).contains(&v),
                "Value {} out of expected range [-0.5, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_pink_noise_output_range() {
        let noise = PinkNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pink_noise_different_seeds() {
        let noise1 = PinkNoise::with_seed(1);
        let noise2 = PinkNoise::with_seed(2);
        let ctx = SignalContext::new(50, 0);

        let v1 = noise1.sample_with_context(0.0, &ctx);
        let v2 = noise2.sample_with_context(0.0, &ctx);

        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_pink_noise_offset() {
        let noise = PinkNoise::new(42, 0.5, 0.25);

        let mut values = Vec::new();
        for frame in 0..100 {
            let ctx = SignalContext::new(frame, 0);
            values.push(noise.sample_with_context(0.0, &ctx));
        }

        let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
        // Mean should be close to offset (allowing some variance)
        assert!(
            (mean - 0.25).abs() < 0.25,
            "Mean {} should be close to offset 0.25",
            mean
        );
    }

    #[test]
    fn test_pink_noise_temporal_continuity() {
        let noise = PinkNoise::with_seed(42);

        // Verify that adjacent frames have some correlation (not pure white noise)
        let mut deltas = Vec::new();
        let mut prev = noise.sample_with_context(0.0, &SignalContext::new(0, 0));

        for frame in 1..100 {
            let curr = noise.sample_with_context(0.0, &SignalContext::new(frame, 0));
            deltas.push((curr - prev).abs());
            prev = curr;
        }

        let avg_delta: f32 = deltas.iter().sum::<f32>() / deltas.len() as f32;
        // Pink noise should have smoother changes than pure white noise
        // Average delta should be moderate (not too large, not zero)
        assert!(
            avg_delta > 0.1 && avg_delta < 1.0,
            "Pink noise avg delta {} should be moderate",
            avg_delta
        );
    }

    #[test]
    fn test_pink_noise_finite() {
        let noise = PinkNoise::with_seed(42);

        for frame in 0..1000 {
            let ctx = SignalContext::new(frame, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }

    #[test]
    fn test_pink_noise_mean_centered() {
        // With bipolar output, mean should be close to offset (0.0 by default)
        let noise = PinkNoise::new(42, 1.0, 0.0);

        let mut sum = 0.0;
        let n = 1000;
        for frame in 0..n {
            let ctx = SignalContext::new(frame, 0);
            sum += noise.sample_with_context(0.0, &ctx);
        }

        let mean = sum / n as f32;
        // Mean should be close to 0.0 (allowing some variance)
        assert!(mean.abs() < 0.2, "Mean {} should be close to 0.0", mean);
    }
}

// <FILE>mixed-signals/src/random/cls_pink_noise.rs</FILE> - <DESC>1/f noise (pink noise) using stateless multi-octave summation</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
