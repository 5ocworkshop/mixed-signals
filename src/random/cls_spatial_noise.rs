// <FILE>src/random/cls_spatial_noise.rs</FILE> - <DESC>Position-based deterministic noise generator</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Noise helpers refactor</WCTX>
// <CLOG>Use core::noise_helpers for u64_to_bipolar; keep custom spatial seeding</CLOG>

use crate::core::u64_to_bipolar;
use crate::math::{finite_or, finite_or_f64, finite_or_min};
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Spatial noise generator producing deterministic noise based on spatial coordinates.
///
/// Unlike time-based noise, this generates values based on (x, y) position.
/// Same position always produces the same value, regardless of time.
/// Useful for per-character effects where character index serves as spatial position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Spatial frequency (scale factor)
    frequency: f32,
    /// Output amplitude
    amplitude: f32,
}

impl SpatialNoise {
    pub fn new(seed: u64, frequency: f32, amplitude: f32) -> Self {
        Self {
            seed,
            frequency: frequency.max(0.01), // Prevent division by zero
            amplitude,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed, 1.0, 1.0)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }
}

impl Default for SpatialNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            frequency: 1.0,
            amplitude: 1.0,
        }
    }
}

/// Generate a deterministic seed for spatial coordinates using ChaCha8Rng
fn derive_spatial_seed(base_seed: u64, x: i32, y: i32) -> [u8; 32] {
    // Combine base seed with spatial coordinates
    let mut h = base_seed;
    h ^= (x as u64).wrapping_mul(0x517cc1b727220a95);
    h = h.rotate_left(31);
    h ^= (y as u64).wrapping_mul(0x9e3779b97f4a7c15);
    h = h.rotate_left(31);
    h ^= h >> 32;
    h = h.wrapping_mul(0x517cc1b727220a95);
    h ^= h >> 32;

    let mut seed_bytes = [0u8; 32];
    for (i, byte) in seed_bytes.iter_mut().enumerate() {
        *byte = (h.wrapping_shr((i * 8) as u32) & 0xFF) as u8;
    }
    seed_bytes
}

impl Signal for SpatialNoise {
    fn output_range(&self) -> SignalRange {
        let amplitude = finite_or(self.amplitude, 1.0);
        SignalRange::new(-amplitude, amplitude)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let frequency = finite_or_min(self.frequency, 0.01, 1.0);
        let amplitude = finite_or(self.amplitude, 1.0);

        // In absence of spatial context, use time as x-coordinate
        let x = (t * frequency as f64) as i32;
        let seed_bytes = derive_spatial_seed(self.seed, x, 0);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let bipolar = u64_to_bipolar(rng.next_u64());
        bipolar as f32 * amplitude
    }

    fn sample_with_context(&self, _t: SignalTime, ctx: &SignalContext) -> f32 {
        let amplitude = finite_or(self.amplitude, 1.0);

        // Use width and height from context for spatial coordinates
        let x = ctx.width as i32;
        let y = ctx.height as i32;

        let effective_seed = self.seed.wrapping_add(ctx.seed);
        let seed_bytes = derive_spatial_seed(effective_seed, x, y);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let bipolar = u64_to_bipolar(rng.next_u64());
        bipolar as f32 * amplitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_noise_determinism() {
        let noise = SpatialNoise::with_seed(42);
        let ctx = SignalContext::new(0, 0).with_dimensions(10, 5);
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(0.0, &ctx);
        assert_eq!(v1, v2, "Same spatial position should produce same value");
    }

    #[test]
    fn test_spatial_noise_different_positions() {
        let noise = SpatialNoise::with_seed(42);
        let ctx1 = SignalContext::new(0, 0).with_dimensions(10, 5);
        let ctx2 = SignalContext::new(0, 0).with_dimensions(15, 5);
        let v1 = noise.sample_with_context(0.0, &ctx1);
        let v2 = noise.sample_with_context(0.0, &ctx2);
        assert_ne!(
            v1, v2,
            "Different positions should produce different values"
        );
    }

    #[test]
    fn test_spatial_noise_time_independent() {
        let noise = SpatialNoise::with_seed(42);
        let ctx = SignalContext::new(0, 0).with_dimensions(10, 5);
        let v1 = noise.sample_with_context(0.0, &ctx);
        let v2 = noise.sample_with_context(100.0, &ctx);
        assert_eq!(
            v1, v2,
            "Same position should produce same value regardless of time"
        );
    }

    #[test]
    fn test_spatial_noise_range() {
        // amplitude=0.5 -> range [-0.5, 0.5]
        let noise = SpatialNoise::new(42, 1.0, 0.5);
        let ctx = SignalContext::new(0, 0).with_dimensions(10, 5);
        let v = noise.sample_with_context(0.0, &ctx);
        assert!(
            (-0.5..=0.5).contains(&v),
            "Value {} out of expected range [-0.5, 0.5]",
            v
        );
    }

    #[test]
    fn test_spatial_noise_output_range() {
        let noise = SpatialNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_noise_different_seeds() {
        let noise1 = SpatialNoise::with_seed(1);
        let noise2 = SpatialNoise::with_seed(2);
        let ctx = SignalContext::new(0, 0).with_dimensions(10, 5);
        let v1 = noise1.sample_with_context(0.0, &ctx);
        let v2 = noise2.sample_with_context(0.0, &ctx);
        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    #[test]
    fn test_spatial_noise_finite() {
        let noise = SpatialNoise::with_seed(42);
        for x in 0..100 {
            for y in 0..10 {
                let ctx = SignalContext::new(0, 0).with_dimensions(x, y);
                let v = noise.sample_with_context(0.0, &ctx);
                assert!(v.is_finite(), "Value must be finite, got {}", v);
            }
        }
    }
}

// <FILE>src/random/cls_spatial_noise.rs</FILE> - <DESC>Position-based deterministic noise generator</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
