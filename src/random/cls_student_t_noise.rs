// <FILE>src/random/cls_student_t_noise.rs</FILE> - <DESC>Student-t distribution noise generator</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for bipolar_range, rng_from_time, rng_from_context</CLOG>

use crate::core::{bipolar_range, rng_from_context, rng_from_time};
use crate::math::finite_or;
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand_distr::{Distribution, StudentT};
use serde::{Deserialize, Serialize};

/// Student-t distribution noise generator.
///
/// Produces random values with heavier tails than Gaussian distribution.
/// This creates more pronounced peaks with less RMS cost, suitable for
/// percussive/impactful audio textures.
///
/// # Degrees of Freedom (df)
///
/// - df = 1: Cauchy distribution (very heavy tails, may be too aggressive)
/// - df = 3: Good balance for audio (recommended)
/// - df = 5-10: Mild heavy tails
/// - df > 30: Approaches Gaussian distribution
///
/// Output is bipolar [-amplitude, +amplitude] centered at offset.
///
/// # Examples
///
/// ```
/// use mixed_signals::random::StudentTNoise;
/// use mixed_signals::traits::Signal;
///
/// let noise = StudentTNoise::default_audio(42);
/// let v = noise.sample(0.5);
/// assert!(v >= -1.0 && v <= 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StudentTNoise {
    /// Degrees of freedom (controls tail heaviness)
    degrees_of_freedom: f32,
    /// Seed for reproducible randomness
    seed: u64,
    /// Scale factor for the distribution
    scale: f32,
    /// Output amplitude
    amplitude: f32,
    /// Center value
    offset: f32,
}

impl StudentTNoise {
    /// Create a new Student-t noise generator.
    ///
    /// # Arguments
    /// * `degrees_of_freedom` - Must be > 0. Lower values = heavier tails.
    /// * `seed` - Seed for reproducibility
    /// * `scale` - Scale factor (default 1.0)
    /// * `amplitude` - Output amplitude (default 1.0)
    /// * `offset` - Output offset (default 0.0)
    ///
    /// # Errors
    /// Returns error if degrees_of_freedom <= 0 or is not finite.
    pub fn new(
        degrees_of_freedom: f32,
        seed: u64,
        scale: f32,
        amplitude: f32,
        offset: f32,
    ) -> Result<Self, String> {
        if degrees_of_freedom <= 0.0 {
            return Err(format!(
                "StudentTNoise degrees_of_freedom must be > 0, got {}",
                degrees_of_freedom
            ));
        }
        if !degrees_of_freedom.is_finite() {
            return Err(format!(
                "StudentTNoise degrees_of_freedom must be finite, got {}",
                degrees_of_freedom
            ));
        }
        if !scale.is_finite() {
            return Err(format!("StudentTNoise scale must be finite, got {}", scale));
        }
        if !amplitude.is_finite() {
            return Err(format!(
                "StudentTNoise amplitude must be finite, got {}",
                amplitude
            ));
        }
        if !offset.is_finite() {
            return Err(format!(
                "StudentTNoise offset must be finite, got {}",
                offset
            ));
        }

        Ok(Self {
            degrees_of_freedom,
            seed,
            scale,
            amplitude,
            offset,
        })
    }

    /// Create a Student-t noise generator with default audio parameters.
    ///
    /// Uses df=3.0 (good heavy tails for audio), scale=1.0, amplitude=1.0, offset=0.0.
    pub fn default_audio(seed: u64) -> Self {
        Self {
            degrees_of_freedom: 3.0,
            seed,
            scale: 1.0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }

    /// Create with just seed, using safe defaults.
    pub fn with_seed(seed: u64) -> Self {
        Self::default_audio(seed)
    }

    pub fn degrees_of_freedom(&self) -> f32 {
        self.degrees_of_freedom
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }
}

impl Default for StudentTNoise {
    fn default() -> Self {
        Self {
            degrees_of_freedom: 3.0,
            seed: 0,
            scale: 1.0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for StudentTNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let df = finite_or(self.degrees_of_freedom, 3.0);
        let scale = finite_or(self.scale, 1.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if df <= 0.0 {
            return offset;
        }

        let mut rng = rng_from_time(self.seed, t);

        let student_t = match StudentT::new(df as f64) {
            Ok(dist) => dist,
            Err(_) => return offset,
        };

        let sample = student_t.sample(&mut rng) as f32;
        if !sample.is_finite() {
            return offset;
        }

        // Normalize using tanh to bound to approximately [-1, 1]
        // Divide by 3 to make typical values span more of the range
        let bipolar = (sample * scale / 3.0).tanh();

        offset + amplitude * bipolar
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let df = finite_or(self.degrees_of_freedom, 3.0);
        let scale = finite_or(self.scale, 1.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if df <= 0.0 {
            return offset;
        }

        let mut rng = rng_from_context(self.seed, t, ctx);

        let student_t = match StudentT::new(df as f64) {
            Ok(dist) => dist,
            Err(_) => return offset,
        };

        let sample = student_t.sample(&mut rng) as f32;
        if !sample.is_finite() {
            return offset;
        }

        // Normalize using tanh to bound to approximately [-1, 1]
        let bipolar = (sample * scale / 3.0).tanh();

        offset + amplitude * bipolar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_student_t_noise_determinism() {
        let noise = StudentTNoise::with_seed(12345);
        let v1 = noise.sample(0.5);
        let v2 = noise.sample(0.5);
        assert_eq!(v1, v2, "Same time should produce same value");
    }

    #[test]
    fn test_student_t_noise_different_times() {
        let noise = StudentTNoise::with_seed(42);
        let v1 = noise.sample(0.0);
        let v2 = noise.sample(0.001);
        // Different times should produce different values (with high probability)
        assert_ne!(v1, v2, "Different times should produce different values");
    }

    #[test]
    fn test_student_t_noise_different_seeds() {
        let noise1 = StudentTNoise::with_seed(1);
        let noise2 = StudentTNoise::with_seed(2);
        let v1 = noise1.sample(0.5);
        let v2 = noise2.sample(0.5);
        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_student_t_noise_range() {
        // Default: amplitude=1.0, offset=0.0 -> range [-1, 1]
        let noise = StudentTNoise::with_seed(42);
        for i in 0..1000 {
            let t = i as f64 * 0.001;
            let v = noise.sample(t);
            assert!(
                (-1.0..=1.0).contains(&v),
                "Value {} should be in range [-1.0, 1.0]",
                v
            );
        }
    }

    #[test]
    fn test_student_t_noise_output_range() {
        let noise = StudentTNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_student_t_noise_mean() {
        // Bipolar: mean should be near 0.0 (the offset)
        let noise = StudentTNoise::with_seed(42);
        let mut sum = 0.0;
        let n = 1000;
        for i in 0..n {
            let t = i as f64 * 0.001;
            sum += noise.sample(t);
        }
        let mean = sum / n as f32;
        // Mean should be close to 0.0
        assert!(mean.abs() < 0.15, "Mean {} should be close to 0.0", mean);
    }

    #[test]
    fn test_student_t_noise_validation() {
        assert!(StudentTNoise::new(0.0, 42, 1.0, 1.0, 0.0).is_err());
        assert!(StudentTNoise::new(-1.0, 42, 1.0, 1.0, 0.0).is_err());
        assert!(StudentTNoise::new(f32::NAN, 42, 1.0, 1.0, 0.0).is_err());
        assert!(StudentTNoise::new(f32::INFINITY, 42, 1.0, 1.0, 0.0).is_err());
        assert!(StudentTNoise::new(3.0, 42, 1.0, f32::NAN, 0.0).is_err());
        assert!(StudentTNoise::new(3.0, 42, 1.0, 1.0, f32::NAN).is_err());
    }

    #[test]
    fn test_student_t_noise_valid_construction() {
        assert!(StudentTNoise::new(1.0, 42, 1.0, 1.0, 0.0).is_ok());
        assert!(StudentTNoise::new(3.0, 42, 1.0, 1.0, 0.0).is_ok());
        assert!(StudentTNoise::new(30.0, 42, 1.0, 1.0, 0.0).is_ok());
    }

    #[test]
    fn test_student_t_noise_finite() {
        let noise = StudentTNoise::with_seed(42);
        for i in 0..1000 {
            let t = i as f64 * 0.1;
            let v = noise.sample(t);
            assert!(v.is_finite(), "Value must be finite, got {}", v);
        }
    }

    #[test]
    fn test_student_t_noise_with_context() {
        let noise = StudentTNoise::with_seed(42);
        let ctx = SignalContext::new(100, 999);
        let v = noise.sample_with_context(0.5, &ctx);
        assert!((-1.0..=1.0).contains(&v));
        assert!(v.is_finite());
    }

    #[test]
    fn test_student_t_noise_context_affects_output() {
        let noise = StudentTNoise::with_seed(42);
        let ctx1 = SignalContext::new(100, 1);
        let ctx2 = SignalContext::new(100, 2);
        let v1 = noise.sample_with_context(0.5, &ctx1);
        let v2 = noise.sample_with_context(0.5, &ctx2);
        assert_ne!(v1, v2, "Different contexts should produce different values");
    }

    #[test]
    fn test_student_t_noise_offset() {
        // amplitude=0.5, offset=0.25 -> range [-0.25, 0.75]
        let noise = StudentTNoise::new(3.0, 42, 1.0, 0.5, 0.25).unwrap();
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

// <FILE>mixed-signals/src/random/cls_student_t_noise.rs</FILE> - <DESC>Student-t distribution noise generator</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
