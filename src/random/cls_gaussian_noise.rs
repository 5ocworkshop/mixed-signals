// <FILE>src/random/cls_gaussian_noise.rs</FILE> - <DESC>Gaussian (normal) distribution noise generator</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, rng_from_time, rng_from_context</CLOG>

use crate::core::{bipolar_range, rng_from_context, rng_from_time};
use crate::math::finite_or;
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

/// Gaussian (normal) distribution noise generator.
///
/// Produces random values with a bell curve distribution. More realistic than
/// uniform randomness for natural phenomena. Values cluster around the center
/// with decreasing probability further away.
///
/// Output is bipolar [-amplitude, +amplitude] centered at offset.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GaussianNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Standard deviation (controls distribution spread)
    std_dev: f32,
    /// Output amplitude (scales the normalized output)
    amplitude: f32,
    /// Center value (shifts the output)
    offset: f32,
}

impl GaussianNoise {
    pub fn new(seed: u64, std_dev: f32, amplitude: f32, offset: f32) -> Result<Self, String> {
        if std_dev < 0.0 {
            return Err(format!(
                "GaussianNoise std_dev must be >= 0, got {}",
                std_dev
            ));
        }
        if !std_dev.is_finite() {
            return Err(format!(
                "GaussianNoise std_dev must be finite, got {}",
                std_dev
            ));
        }
        if !amplitude.is_finite() {
            return Err(format!(
                "GaussianNoise amplitude must be finite, got {}",
                amplitude
            ));
        }
        if !offset.is_finite() {
            return Err(format!(
                "GaussianNoise offset must be finite, got {}",
                offset
            ));
        }
        Ok(Self {
            seed,
            std_dev,
            amplitude,
            offset,
        })
    }

    /// Convenience constructor with default parameters.
    ///
    /// Uses std_dev=1.0, amplitude=1.0, offset=0.0 (always valid).
    pub fn with_seed(seed: u64) -> Self {
        // Safe: defaults are always valid
        Self::new(seed, 1.0, 1.0, 0.0).unwrap()
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn std_dev(&self) -> f32 {
        self.std_dev
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }
}

impl Default for GaussianNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            std_dev: 1.0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for GaussianNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let std_dev = finite_or(self.std_dev, 1.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if std_dev <= 0.0 {
            return offset;
        }

        let mut rng = rng_from_time(self.seed, t);

        // Sample from standard normal, then scale by std_dev
        let normal = match Normal::new(0.0, std_dev as f64) {
            Ok(normal) => normal,
            Err(_) => return offset,
        };
        let value = normal.sample(&mut rng) as f32;
        if !value.is_finite() {
            return offset;
        }

        // Normalize to [-1, 1] using 3-sigma rule (99.7% coverage)
        let bound = 3.0 * std_dev;
        let bipolar = (value / bound).clamp(-1.0, 1.0);

        offset + amplitude * bipolar
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let std_dev = finite_or(self.std_dev, 1.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if std_dev <= 0.0 {
            return offset;
        }

        let mut rng = rng_from_context(self.seed, t, ctx);

        // Sample from standard normal, then scale by std_dev
        let normal = match Normal::new(0.0, std_dev as f64) {
            Ok(normal) => normal,
            Err(_) => return offset,
        };
        let value = normal.sample(&mut rng) as f32;
        if !value.is_finite() {
            return offset;
        }

        // Normalize to [-1, 1] using 3-sigma rule (99.7% coverage)
        let bound = 3.0 * std_dev;
        let bipolar = (value / bound).clamp(-1.0, 1.0);

        offset + amplitude * bipolar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_noise_determinism() {
        let noise = GaussianNoise::with_seed(12345);
        let v1 = noise.sample(0.5);
        let v2 = noise.sample(0.5);
        assert_eq!(v1, v2, "Same time should produce same value");
    }

    #[test]
    fn test_gaussian_noise_mean() {
        // Bipolar: mean should be near 0.0 (the offset)
        let noise = GaussianNoise::with_seed(42);
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
    fn test_gaussian_noise_range() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = GaussianNoise::new(42, 1.0, 0.5, 0.0).unwrap();
        for i in 0..1000 {
            let t = i as f64 * 0.01;
            let v = noise.sample(t);
            assert!(
                (-0.5..=0.5).contains(&v),
                "Value {} should be in range [-0.5, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_gaussian_noise_output_range() {
        let noise = GaussianNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gaussian_noise_validation() {
        // Negative std_dev should fail
        assert!(GaussianNoise::new(42, -0.5, 1.0, 0.0).is_err());

        // Non-finite values should fail
        assert!(GaussianNoise::new(42, f32::NAN, 1.0, 0.0).is_err());
        assert!(GaussianNoise::new(42, f32::INFINITY, 1.0, 0.0).is_err());
        assert!(GaussianNoise::new(42, 1.0, f32::NAN, 0.0).is_err());
        assert!(GaussianNoise::new(42, 1.0, 1.0, f32::NAN).is_err());
    }

    #[test]
    fn test_gaussian_noise_zero_std_dev() {
        // With zero std_dev, should return offset
        let noise = GaussianNoise::new(42, 0.0, 1.0, 0.25).unwrap();
        for i in 0..10 {
            let t = i as f64 * 0.1;
            let v = noise.sample(t);
            assert_eq!(v, 0.25, "Zero std_dev should always return offset");
        }
    }

    #[test]
    fn test_gaussian_noise_invalid_std_dev_no_panic() {
        let noise = GaussianNoise {
            seed: 1,
            std_dev: -1.0,
            amplitude: 1.0,
            offset: 0.5,
        };
        let v = noise.sample(0.25);
        assert_eq!(v, 0.5);
    }

    #[test]
    fn test_gaussian_noise_nan_params_no_panic() {
        let noise = GaussianNoise {
            seed: 1,
            std_dev: f32::NAN,
            amplitude: f32::NAN,
            offset: f32::NAN,
        };
        let v = noise.sample(0.25);
        assert!(v.is_finite());
        // With NAN params, defaults apply: std_dev=1.0, amplitude=1.0, offset=0.0
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn test_gaussian_noise_different_seeds() {
        let noise1 = GaussianNoise::with_seed(1);
        let noise2 = GaussianNoise::with_seed(2);
        let v1 = noise1.sample(0.5);
        let v2 = noise2.sample(0.5);
        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_gaussian_noise_finite() {
        let noise = GaussianNoise::with_seed(42);
        for i in 0..1000 {
            let t = i as f64 * 0.1;
            let v = noise.sample(t);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }

    #[test]
    fn test_gaussian_noise_offset() {
        // amplitude=0.5, offset=0.25 -> range [-0.25, 0.75]
        let noise = GaussianNoise::new(42, 1.0, 0.5, 0.25).unwrap();
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

// <FILE>mixed-signals/src/random/cls_gaussian_noise.rs</FILE> - <DESC>Gaussian (normal) distribution noise generator</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
