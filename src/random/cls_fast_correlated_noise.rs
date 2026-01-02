// <FILE>src/random/cls_fast_correlated_noise.rs</FILE> - <DESC>Fast temporally correlated noise using hash-based RNG</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, ema_smoothing, scale_bipolar</CLOG>

use crate::core::{bipolar_range, ema_smoothing, scale_bipolar};
use crate::math::fast_random;
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Fast correlated noise using hash-based RNG.
///
/// ~25x faster than `CorrelatedNoise` per lookback frame by using SplitMix64.
/// With 10 lookback frames, this is ~250x faster overall.
/// Suitable for animation/visualization where cryptographic quality isn't needed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FastCorrelatedNoise {
    seed: u64,
    correlation: f32,
    amplitude: f32,
    offset: f32,
}

impl FastCorrelatedNoise {
    pub fn new(seed: u64, correlation: f32, amplitude: f32, offset: f32) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&correlation) {
            return Err(format!(
                "FastCorrelatedNoise correlation must be 0.0-1.0, got {}",
                correlation
            ));
        }
        Ok(Self {
            seed,
            correlation,
            amplitude,
            offset,
        })
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed, 0.95, 1.0, 0.0).unwrap()
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn correlation(&self) -> f32 {
        self.correlation
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }
}

impl Default for FastCorrelatedNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            correlation: 0.95,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for FastCorrelatedNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let frame = (crate::math::finite_or_f64(t, 0.0) * 60.0) as u64;
        let seed = self.seed;
        let bipolar = ema_smoothing(frame, self.correlation, 10, |past_frame| {
            // fast_random returns [0, 1], convert to bipolar [-1, 1]
            fast_random(seed, past_frame) * 2.0 - 1.0
        });
        scale_bipolar(bipolar as f64, self.amplitude, self.offset)
    }

    fn sample_with_context(&self, _t: SignalTime, ctx: &SignalContext) -> f32 {
        let effective_seed = self.seed.wrapping_add(ctx.seed);
        let bipolar = ema_smoothing(ctx.frame, self.correlation, 10, |past_frame| {
            // fast_random returns [0, 1], convert to bipolar [-1, 1]
            fast_random(effective_seed, past_frame) * 2.0 - 1.0
        });
        scale_bipolar(bipolar as f64, self.amplitude, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_correlated_noise_determinism() {
        let noise = FastCorrelatedNoise::with_seed(12345);
        let ctx = SignalContext::new(100, 0);
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_fast_correlated_noise_smoothness() {
        let noise = FastCorrelatedNoise::new(42, 0.95, 1.0, 0.0).unwrap();
        let mut deltas = Vec::new();
        let mut prev = noise.sample_with_context(0.0, &SignalContext::new(0, 0));

        for frame in 1..100 {
            let curr = noise.sample_with_context(0.0, &SignalContext::new(frame, 0));
            deltas.push((curr - prev).abs());
            prev = curr;
        }

        let avg_delta: f32 = deltas.iter().sum::<f32>() / deltas.len() as f32;
        assert!(
            avg_delta < 0.5,
            "High correlation should produce smooth changes, got {}",
            avg_delta
        );
    }

    #[test]
    fn test_fast_correlated_noise_validation() {
        assert!(FastCorrelatedNoise::new(42, -0.1, 1.0, 0.0).is_err());
        assert!(FastCorrelatedNoise::new(42, 1.1, 1.0, 0.0).is_err());
        assert!(FastCorrelatedNoise::new(42, 0.5, 1.0, 0.0).is_ok());
    }

    #[test]
    fn test_fast_correlated_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = FastCorrelatedNoise::new(42, 0.95, 0.5, 0.0).unwrap();
        for frame in 0..100 {
            let ctx = SignalContext::new(frame, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!((-0.5..=0.5).contains(&v), "Value {} out of range", v);
        }
    }

    #[test]
    fn test_fast_correlated_noise_output_range() {
        let noise = FastCorrelatedNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }
}

// <FILE>src/random/cls_fast_correlated_noise.rs</FILE> - <DESC>Fast temporally correlated noise using hash-based RNG</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
