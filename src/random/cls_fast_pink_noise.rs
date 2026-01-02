// <FILE>src/random/cls_fast_pink_noise.rs</FILE> - <DESC>Fast 1/f noise using hash-based RNG</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, octave_sum, scale_bipolar</CLOG>

use crate::core::{bipolar_range, octave_sum, scale_bipolar};
use crate::math::{fast_random, finite_or_f64};
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Fast pink noise (1/f noise) using hash-based RNG.
///
/// ~25x faster than `PinkNoise` per octave by using SplitMix64 mixing.
/// With 5 octaves, this is ~125x faster overall.
/// Suitable for animation/visualization where cryptographic quality isn't needed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FastPinkNoise {
    seed: u64,
    amplitude: f32,
    offset: f32,
}

impl FastPinkNoise {
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

impl Default for FastPinkNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for FastPinkNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let frame = (finite_or_f64(t, 0.0) * 60.0) as u64;
        let bipolar = octave_sum(self.seed, frame, 5, |octave_seed, octave_frame| {
            // fast_random returns [0, 1], convert to bipolar [-1, 1]
            fast_random(octave_seed, octave_frame) * 2.0 - 1.0
        });
        scale_bipolar(bipolar as f64, self.amplitude, self.offset)
    }

    fn sample_with_context(&self, _t: SignalTime, ctx: &SignalContext) -> f32 {
        let effective_seed = self.seed.wrapping_add(ctx.seed);
        let bipolar = octave_sum(effective_seed, ctx.frame, 5, |octave_seed, octave_frame| {
            // fast_random returns [0, 1], convert to bipolar [-1, 1]
            fast_random(octave_seed, octave_frame) * 2.0 - 1.0
        });
        scale_bipolar(bipolar as f64, self.amplitude, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_pink_noise_determinism() {
        let noise = FastPinkNoise::with_seed(12345);
        let ctx = SignalContext::new(100, 0);
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_fast_pink_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = FastPinkNoise::new(42, 0.5, 0.0);
        for frame in 0..100 {
            let ctx = SignalContext::new(frame, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!((-0.5..=0.5).contains(&v), "Value {} out of range", v);
        }
    }

    #[test]
    fn test_fast_pink_noise_output_range() {
        let noise = FastPinkNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fast_pink_noise_different_seeds() {
        let noise1 = FastPinkNoise::with_seed(1);
        let noise2 = FastPinkNoise::with_seed(2);
        let ctx = SignalContext::new(50, 0);
        let v1 = noise1.sample_with_context(0.0, &ctx);
        let v2 = noise2.sample_with_context(0.0, &ctx);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_fast_pink_noise_temporal_continuity() {
        let noise = FastPinkNoise::with_seed(42);
        let mut deltas = Vec::new();
        let mut prev = noise.sample_with_context(0.0, &SignalContext::new(0, 0));

        for frame in 1..100 {
            let curr = noise.sample_with_context(0.0, &SignalContext::new(frame, 0));
            deltas.push((curr - prev).abs());
            prev = curr;
        }

        let avg_delta: f32 = deltas.iter().sum::<f32>() / deltas.len() as f32;
        assert!(
            avg_delta > 0.01 && avg_delta < 1.0,
            "Avg delta {} should be moderate",
            avg_delta
        );
    }

    #[test]
    fn test_fast_pink_noise_mean() {
        // Bipolar: mean should be near 0.0
        let noise = FastPinkNoise::with_seed(42);
        let mut sum = 0.0;
        for frame in 0..1000 {
            let ctx = SignalContext::new(frame, 0);
            sum += noise.sample_with_context(0.0, &ctx);
        }
        let mean = sum / 1000.0;
        assert!(mean.abs() < 0.2, "Mean {} should be near 0.0", mean);
    }
}

// <FILE>src/random/cls_fast_pink_noise.rs</FILE> - <DESC>Fast 1/f noise using hash-based RNG</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
