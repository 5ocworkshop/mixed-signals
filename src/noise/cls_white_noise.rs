// <FILE>mixed-signals/src/noise/cls_white_noise.rs</FILE> - <DESC>White noise generator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor</WCTX>
// <CLOG>Converted to bipolar [-1,1] output, added output_range()</CLOG>

use crate::math::{derive_seed, finite_or, finite_or_f64, finite_or_min};
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// White noise generator producing random values.
///
/// Uses deterministic ChaCha8Rng based on seed and time for reproducible noise.
/// Output is bipolar [-amplitude, +amplitude] centered at offset.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WhiteNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Output amplitude (half of total range)
    amplitude: f32,
    /// Center value (offset)
    offset: f32,
    /// Quantization level (samples per second, higher = more variation)
    sample_rate: f32,
}

impl WhiteNoise {
    pub fn new(seed: u64, amplitude: f32, sample_rate: f32) -> Self {
        Self {
            seed,
            amplitude,
            offset: 0.0,
            sample_rate: sample_rate.max(1.0),
        }
    }

    /// Create with full parameters including offset.
    pub fn with_offset(seed: u64, amplitude: f32, offset: f32, sample_rate: f32) -> Self {
        Self {
            seed,
            amplitude,
            offset,
            sample_rate: sample_rate.max(1.0),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed, 1.0, 60.0)
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

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            amplitude: 1.0,
            offset: 0.0,
            sample_rate: 60.0,
        }
    }
}

impl Signal for WhiteNoise {
    fn output_range(&self) -> SignalRange {
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);
        SignalRange::new(offset - amplitude, offset + amplitude)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);
        let sample_rate = finite_or_min(self.sample_rate, 1.0, 60.0);

        // Quantize time to sample rate
        let sample_index = (t * sample_rate as f64) as u64;
        let seed_bytes = derive_seed(self.seed, sample_index);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let random_value = rng.next_u64();
        // Convert to bipolar [-1, 1] range
        let bipolar = (random_value as f64 / u64::MAX as f64) * 2.0 - 1.0;
        offset + amplitude * bipolar as f32
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);
        let sample_rate = finite_or_min(self.sample_rate, 1.0, 60.0);

        // Use context seed if available
        let effective_seed = if ctx.seed != 0 {
            self.seed.wrapping_add(ctx.seed)
        } else {
            self.seed
        };
        let sample_index = (t * sample_rate as f64) as u64;
        let combined_index = sample_index.wrapping_add(ctx.frame);
        let seed_bytes = derive_seed(effective_seed, combined_index);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let random_value = rng.next_u64();
        // Convert to bipolar [-1, 1] range
        let bipolar = (random_value as f64 / u64::MAX as f64) * 2.0 - 1.0;
        offset + amplitude * bipolar as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_noise_bipolar_bounded() {
        // Default: amplitude=1.0, offset=0.0 -> range [-1, 1]
        let noise = WhiteNoise::default();
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = noise.sample(t);
            assert!((-1.0..=1.0).contains(&v), "Value {} out of bipolar range", v);
        }
    }

    #[test]
    fn test_white_noise_deterministic() {
        let noise = WhiteNoise::with_seed(42);
        let v1 = noise.sample(0.5);
        let v2 = noise.sample(0.5);
        assert!((v1 - v2).abs() < 0.001);
    }

    #[test]
    fn test_white_noise_different_seeds() {
        let noise1 = WhiteNoise::with_seed(1);
        let noise2 = WhiteNoise::with_seed(2);
        // Different seeds should produce different values
        let v1 = noise1.sample(0.5);
        let v2 = noise2.sample(0.5);
        assert!((v1 - v2).abs() > 0.001);
    }

    #[test]
    fn test_white_noise_amplitude() {
        // amplitude=0.5, offset=0 -> range [-0.5, 0.5]
        let noise = WhiteNoise::new(42, 0.5, 60.0);
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = noise.sample(t);
            assert!(
                (-0.5..=0.5).contains(&v),
                "Value {} out of range [-0.5, 0.5]",
                v
            );
        }
    }

    #[test]
    fn test_white_noise_output_range() {
        let noise = WhiteNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);

        let noise2 = WhiteNoise::with_offset(42, 0.5, 0.25, 60.0);
        let range2 = noise2.output_range();
        assert!((range2.min - (-0.25)).abs() < 0.001);
        assert!((range2.max - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_white_noise_with_offset() {
        // amplitude=0.5, offset=0.5 -> range [0, 1]
        let noise = WhiteNoise::with_offset(42, 0.5, 0.5, 60.0);
        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = noise.sample(t);
            assert!((0.0..=1.0).contains(&v), "Value {} out of range [0, 1]", v);
        }
    }
}

// <FILE>mixed-signals/src/noise/cls_white_noise.rs</FILE> - <DESC>White noise generator</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
