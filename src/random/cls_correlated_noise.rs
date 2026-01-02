// <FILE>src/random/cls_correlated_noise.rs</FILE> - <DESC>Temporally smooth correlated noise using stateless RNG-based approach</DESC>
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

/// Correlated noise generator producing temporally smooth random drift.
///
/// Creates smooth random changes over time (Brownian motion / random walk).
/// Uses stateless ChaCha8Rng-based approach with frame lookback window for determinism.
/// Higher correlation (0.0-1.0) produces slower, smoother changes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CorrelatedNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Correlation factor (0.0 = white noise, 1.0 = fully correlated)
    correlation: f32,
    /// Output amplitude
    amplitude: f32,
    /// Center value
    offset: f32,
}

impl CorrelatedNoise {
    pub fn new(seed: u64, correlation: f32, amplitude: f32, offset: f32) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&correlation) {
            return Err(format!(
                "CorrelatedNoise correlation must be 0.0-1.0, got {}",
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

    /// Convenience constructor with default parameters.
    ///
    /// Uses correlation=0.95, amplitude=1.0, offset=0.0 (always valid).
    pub fn with_seed(seed: u64) -> Self {
        // Safe: defaults are always valid
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

impl Default for CorrelatedNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            correlation: 0.95,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for CorrelatedNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let correlation = finite_or(self.correlation, 0.95);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        // Convert time to frame equivalent (assume 60fps for consistency)
        let frame = (t * 60.0) as u64;

        // Exponential moving average using frame-based hashing
        let window = 10; // Lookback window
        let mut smoothed = 0.0;
        let mut weight_sum = 0.0;

        for i in 0..window {
            if frame >= i {
                let past_frame = frame - i;
                let seed_bytes = derive_seed(self.seed, past_frame);
                let mut rng = ChaCha8Rng::from_seed(seed_bytes);
                let random_value = rng.next_u64();
                let bipolar = u64_to_bipolar(random_value);

                let weight = correlation.powi(i as i32);
                smoothed += bipolar as f32 * weight;
                weight_sum += weight;
            }
        }

        if weight_sum > 0.0 {
            offset + (smoothed / weight_sum) * amplitude
        } else {
            offset
        }
    }

    fn sample_with_context(&self, _t: SignalTime, ctx: &SignalContext) -> f32 {
        let correlation = finite_or(self.correlation, 0.95);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        // Use context frame directly for temporal correlation
        let frame = ctx.frame;
        let effective_seed = self.seed.wrapping_add(ctx.seed);

        // Exponential moving average using frame-based hashing
        let window = 10; // Lookback window
        let mut smoothed = 0.0;
        let mut weight_sum = 0.0;

        for i in 0..window {
            if frame >= i {
                let past_frame = frame - i;
                let seed_bytes = derive_seed(effective_seed, past_frame);
                let mut rng = ChaCha8Rng::from_seed(seed_bytes);
                let random_value = rng.next_u64();
                let bipolar = u64_to_bipolar(random_value);

                let weight = correlation.powi(i as i32);
                smoothed += bipolar as f32 * weight;
                weight_sum += weight;
            }
        }

        if weight_sum > 0.0 {
            offset + (smoothed / weight_sum) * amplitude
        } else {
            offset
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlated_noise_determinism() {
        let noise = CorrelatedNoise::with_seed(12345);
        let ctx = SignalContext::new(100, 0);
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        assert_eq!(v1, v2, "Same frame should produce same value (stateless)");
    }

    #[test]
    fn test_correlated_noise_stateless() {
        let noise = CorrelatedNoise::with_seed(42);
        let ctx = SignalContext::new(50, 0);

        // Multiple calls with same context should yield identical output
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        let v3 = noise.sample_with_context(0.0, &ctx);

        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
    }

    #[test]
    fn test_correlated_noise_smoothness() {
        let noise = CorrelatedNoise::new(42, 0.95, 1.0, 0.0).unwrap();

        let mut deltas = Vec::new();
        let mut prev = noise.sample_with_context(0.0, &SignalContext::new(0, 0));

        for frame in 1..100 {
            let curr = noise.sample_with_context(0.0, &SignalContext::new(frame, 0));
            deltas.push((curr - prev).abs());
            prev = curr;
        }

        // Average frame-to-frame change should be small with high correlation
        let avg_delta: f32 = deltas.iter().sum::<f32>() / deltas.len() as f32;
        assert!(
            avg_delta < 0.5,
            "High correlation should produce smooth changes, got avg delta {}",
            avg_delta
        );
    }

    #[test]
    fn test_correlated_noise_correlation_range() {
        // Test that correlation must be 0.0-1.0
        assert!(CorrelatedNoise::new(42, -0.1, 1.0, 0.0).is_err());
        assert!(CorrelatedNoise::new(42, 1.1, 1.0, 0.0).is_err());
        assert!(CorrelatedNoise::new(42, 0.0, 1.0, 0.0).is_ok());
        assert!(CorrelatedNoise::new(42, 1.0, 1.0, 0.0).is_ok());
    }

    #[test]
    fn test_correlated_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = CorrelatedNoise::new(42, 0.95, 0.5, 0.0).unwrap();

        for frame in 0..100 {
            let ctx = SignalContext::new(frame, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!(
                (-0.5..=0.5).contains(&v),
                "Value {} out of expected range [-0.5, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_correlated_noise_output_range() {
        let noise = CorrelatedNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlated_noise_offset() {
        let noise = CorrelatedNoise::new(42, 0.9, 0.5, 0.25).unwrap();

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
    fn test_correlated_noise_nan_params_no_panic() {
        let noise = CorrelatedNoise {
            seed: 1,
            correlation: f32::NAN,
            amplitude: f32::NAN,
            offset: f32::NAN,
        };
        let v = noise.sample(0.25);
        assert!(v.is_finite());
        // With NAN params, defaults apply, so range is [-1, 1]
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn test_correlated_noise_finite() {
        let noise = CorrelatedNoise::with_seed(42);

        for frame in 0..1000 {
            let ctx = SignalContext::new(frame, 0);
            let v = noise.sample_with_context(0.0, &ctx);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }
}

// <FILE>src/random/cls_correlated_noise.rs</FILE> - <DESC>Temporally smooth correlated noise using stateless RNG-based approach</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
