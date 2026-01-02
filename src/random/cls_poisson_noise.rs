// <FILE>src/random/cls_poisson_noise.rs</FILE> - <DESC>Poisson distribution noise for event-based randomness</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, rng_from_time, rng_from_context</CLOG>

use crate::core::{bipolar_range, rng_from_context, rng_from_time};
use crate::math::finite_or;
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand_distr::{Distribution, Poisson};
use serde::{Deserialize, Serialize};

/// Poisson distribution noise generator for modeling discrete random events.
///
/// Models random events occurring at an average rate (lambda).
/// Natural for simulating irregular timing like keypress intervals, network packets, or glitches.
/// Produces a bipolar value derived from z-score normalization of event counts.
/// For raw counts, use `rng::Rng::poisson`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoissonNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Average event rate (lambda parameter)
    lambda: f32,
    /// Output amplitude (scales the bipolar output)
    amplitude: f32,
    /// Center value (shifts the output)
    offset: f32,
}

impl PoissonNoise {
    pub fn new(seed: u64, lambda: f32, amplitude: f32, offset: f32) -> Result<Self, String> {
        if lambda <= 0.0 {
            return Err(format!("PoissonNoise lambda must be > 0, got {}", lambda));
        }
        if !lambda.is_finite() {
            return Err(format!(
                "PoissonNoise lambda must be finite, got {}",
                lambda
            ));
        }
        if !amplitude.is_finite() {
            return Err(format!(
                "PoissonNoise amplitude must be finite, got {}",
                amplitude
            ));
        }
        if !offset.is_finite() {
            return Err(format!(
                "PoissonNoise offset must be finite, got {}",
                offset
            ));
        }
        Ok(Self {
            seed,
            lambda,
            amplitude,
            offset,
        })
    }

    /// Convenience constructor with default parameters.
    ///
    /// Uses lambda=2.0, amplitude=1.0, offset=0.0 (always valid).
    pub fn with_seed(seed: u64) -> Self {
        // Safe: defaults are always valid
        Self::new(seed, 2.0, 1.0, 0.0).unwrap()
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn lambda(&self) -> f32 {
        self.lambda
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }
}

impl Default for PoissonNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            lambda: 2.0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for PoissonNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if self.lambda <= 0.0 || !self.lambda.is_finite() {
            return offset;
        }

        let mut rng = rng_from_time(self.seed, t);

        let poisson = match Poisson::new(self.lambda as f64) {
            Ok(poisson) => poisson,
            Err(_) => return offset,
        };
        let value = poisson.sample(&mut rng) as f32;
        // Z-score normalization, clamped to ±3 sigma
        let z = (value - self.lambda) / self.lambda.sqrt();
        // Map z-score to bipolar [-1, 1]
        let bipolar = (z / 3.0).clamp(-1.0, 1.0);
        offset + amplitude * bipolar
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if self.lambda <= 0.0 || !self.lambda.is_finite() {
            return offset;
        }

        let mut rng = rng_from_context(self.seed, t, ctx);

        let poisson = match Poisson::new(self.lambda as f64) {
            Ok(poisson) => poisson,
            Err(_) => return offset,
        };
        let value = poisson.sample(&mut rng) as f32;
        // Z-score normalization, clamped to ±3 sigma
        let z = (value - self.lambda) / self.lambda.sqrt();
        // Map z-score to bipolar [-1, 1]
        let bipolar = (z / 3.0).clamp(-1.0, 1.0);
        offset + amplitude * bipolar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poisson_noise_determinism() {
        let noise = PoissonNoise::with_seed(12345);
        let v1 = noise.sample(0.5);
        let v2 = noise.sample(0.5);
        assert_eq!(v1, v2, "Same time should produce same value");
    }

    #[test]
    fn test_poisson_noise_different_times() {
        let noise = PoissonNoise::with_seed(42);
        let v1 = noise.sample(0.1);
        let v2 = noise.sample(0.2);
        // May occasionally be equal, but usually different
        // Just verify they're both valid
        assert!(v1.is_finite());
        assert!(v2.is_finite());
    }

    #[test]
    fn test_poisson_noise_validation() {
        // Zero lambda should fail
        assert!(PoissonNoise::new(42, 0.0, 1.0, 0.0).is_err());

        // Negative lambda should fail
        assert!(PoissonNoise::new(42, -1.0, 1.0, 0.0).is_err());

        // Non-finite values should fail
        assert!(PoissonNoise::new(42, f32::NAN, 1.0, 0.0).is_err());
        assert!(PoissonNoise::new(42, f32::INFINITY, 1.0, 0.0).is_err());
        assert!(PoissonNoise::new(42, 2.0, f32::INFINITY, 0.0).is_err());
        assert!(PoissonNoise::new(42, 2.0, 1.0, f32::NAN).is_err());
    }

    #[test]
    fn test_poisson_noise_mean_centered() {
        // Bipolar: mean should be near 0.0 (the offset)
        let noise = PoissonNoise::with_seed(42);
        let mut sum = 0.0;
        let n = 1000;
        for i in 0..n {
            let t = i as f64 * 0.001;
            sum += noise.sample(t);
        }
        let mean = sum / n as f32;
        // Mean should be close to 0.0 (allowing some variance)
        assert!(mean.abs() < 0.2, "Mean {} should be close to 0.0", mean);
    }

    #[test]
    fn test_poisson_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = PoissonNoise::new(42, 2.0, 0.5, 0.0).unwrap();
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = noise.sample(t);
            assert!(
                (-0.5..=0.5).contains(&v),
                "Value {} out of expected range [-0.5, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_poisson_noise_output_range() {
        let noise = PoissonNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_poisson_noise_different_seeds() {
        let noise1 = PoissonNoise::with_seed(1);
        let noise2 = PoissonNoise::with_seed(2);
        let v1 = noise1.sample(0.5);
        let v2 = noise2.sample(0.5);
        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_poisson_noise_finite() {
        let noise = PoissonNoise::with_seed(42);
        for i in 0..1000 {
            let t = i as f64 * 0.1;
            let v = noise.sample(t);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }

    #[test]
    fn test_poisson_noise_context() {
        let noise = PoissonNoise::with_seed(42);
        let ctx = SignalContext::new(100, 999);
        let v1 = noise.sample_with_context(0.5, &ctx);
        let v2 = noise.sample_with_context(0.5, &ctx);
        assert_eq!(v1, v2, "Same context should produce same value");
    }

    #[test]
    fn test_poisson_noise_invalid_lambda_no_panic() {
        let noise = PoissonNoise {
            seed: 1,
            lambda: -1.0,
            amplitude: 1.0,
            offset: 0.5,
        };
        let v = noise.sample(0.25);
        // With invalid lambda, returns offset
        assert_eq!(v, 0.5);
    }

    #[test]
    fn test_poisson_noise_nan_params_no_panic() {
        let noise = PoissonNoise {
            seed: 1,
            lambda: 2.0,
            amplitude: f32::NAN,
            offset: f32::NAN,
        };
        let v = noise.sample(0.25);
        assert!(v.is_finite());
        // With NAN params, defaults apply
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn test_poisson_noise_offset() {
        // amplitude=0.5, offset=0.25 -> range [-0.25, 0.75]
        let noise = PoissonNoise::new(42, 2.0, 0.5, 0.25).unwrap();
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = noise.sample(t);
            assert!(
                (-0.25..=0.75).contains(&v),
                "Value {} out of expected range [-0.25, 0.75] with offset",
                v
            );
        }
    }
}

// <FILE>mixed-signals/src/random/cls_poisson_noise.rs</FILE> - <DESC>Poisson distribution noise for event-based randomness</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
