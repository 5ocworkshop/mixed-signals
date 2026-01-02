// <FILE>mixed-signals/src/noise/cls_perlin.rs</FILE> - <DESC>Perlin-like smooth noise generator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor</WCTX>
// <CLOG>Converted to bipolar [-1,1] output, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Smooth coherent noise generator (simplified Perlin-like).
///
/// Produces smooth, continuous noise by interpolating between
/// random values at integer time points.
/// Output is bipolar [-amplitude, +amplitude] centered at offset.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PerlinNoise {
    /// Seed for reproducible randomness
    seed: u64,
    /// Scale factor (higher = more compressed/faster variation)
    scale: f32,
    /// Output amplitude (half of total range)
    amplitude: f32,
    /// Center value (offset)
    offset: f32,
    /// Number of octaves for fractal noise (1 = simple, more = detailed)
    octaves: u8,
    /// Persistence for octave amplitude decay (typically 0.5)
    persistence: f32,
}

impl PerlinNoise {
    pub fn new(seed: u64, scale: f32, amplitude: f32) -> Self {
        Self {
            seed,
            scale,
            amplitude,
            offset: 0.0,
            octaves: 1,
            persistence: 0.5,
        }
    }

    /// Create with full parameters including offset.
    pub fn with_offset(seed: u64, scale: f32, amplitude: f32, offset: f32) -> Self {
        Self {
            seed,
            scale,
            amplitude,
            offset,
            octaves: 1,
            persistence: 0.5,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed, 1.0, 1.0)
    }

    pub fn with_octaves(mut self, octaves: u8, persistence: f32) -> Self {
        self.octaves = octaves.max(1);
        self.persistence = persistence;
        self
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

    pub fn octaves(&self) -> u8 {
        self.octaves
    }

    pub fn persistence(&self) -> f32 {
        self.persistence
    }
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self {
            seed: 0,
            scale: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            octaves: 1,
            persistence: 0.5,
        }
    }
}

/// Hash function for gradient generation
fn hash(seed: u64, x: i64) -> f64 {
    let mut n = seed.wrapping_add(x as u64).wrapping_mul(0x517cc1b727220a95);
    n ^= n >> 32;
    n = n.wrapping_mul(0x9e3779b97f4a7c15);
    n ^= n >> 32;
    // Return value in -1..1
    (n as f64 / u64::MAX as f64) * 2.0 - 1.0
}

/// Smoothstep interpolation (ease in-out)
fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Single octave of noise
fn noise_1d(seed: u64, x: f64) -> f64 {
    let x0 = x.floor() as i64;
    let x1 = x0 + 1;
    let t = x - x.floor();
    let t_smooth = smoothstep(t);

    let g0 = hash(seed, x0);
    let g1 = hash(seed, x1);

    lerp(g0, g1, t_smooth)
}

impl Signal for PerlinNoise {
    fn output_range(&self) -> SignalRange {
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);
        SignalRange::new(offset - amplitude, offset + amplitude)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let scale = finite_or(self.scale, 1.0) as f64;
        let amplitude_scale = finite_or(self.amplitude, 1.0) as f64;
        let offset = finite_or(self.offset, 0.0) as f64;
        let persistence = finite_or(self.persistence, 0.5) as f64;
        let octaves = self.octaves.max(1);

        let mut total: f64 = 0.0;
        let mut frequency = scale;
        let mut amplitude: f64 = 1.0;
        let mut max_value: f64 = 0.0;

        for i in 0..octaves {
            let octave_seed = self.seed.wrapping_add(i as u64 * 31337);
            total += noise_1d(octave_seed, t * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        // noise_1d returns [-1, 1], total/max_value is also [-1, 1]
        // Apply amplitude and offset for bipolar output
        let bipolar = total / max_value;
        (offset + bipolar * amplitude_scale) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_bipolar_bounded() {
        // Default: amplitude=1.0, offset=0.0 -> range [-1, 1]
        let noise = PerlinNoise::default();
        for i in 0..100 {
            let t = i as f64 * 0.1;
            let v = noise.sample(t);
            assert!(
                (-1.0..=1.0).contains(&v),
                "Value {} out of bipolar range at t={}",
                v,
                t
            );
        }
    }

    #[test]
    fn test_perlin_smooth() {
        let noise = PerlinNoise::with_seed(42);
        // Adjacent samples should be relatively close (smooth)
        let v1 = noise.sample(0.5);
        let v2 = noise.sample(0.51);
        assert!((v1 - v2).abs() < 0.5, "Values {} and {} not smooth", v1, v2);
    }

    #[test]
    fn test_perlin_deterministic() {
        let noise = PerlinNoise::with_seed(42);
        let v1 = noise.sample(0.5);
        let v2 = noise.sample(0.5);
        assert!((v1 - v2).abs() < 0.001);
    }

    #[test]
    fn test_perlin_octaves() {
        let simple = PerlinNoise::with_seed(42);
        let fractal = PerlinNoise::with_seed(42).with_octaves(4, 0.5);
        // Both should be bounded to bipolar range
        for i in 0..50 {
            let t = i as f64 * 0.1;
            let v1 = simple.sample(t);
            let v2 = fractal.sample(t);
            assert!((-1.0..=1.0).contains(&v1));
            assert!((-1.0..=1.0).contains(&v2));
        }
    }

    #[test]
    fn test_perlin_octaves_zero_safe() {
        let noise = PerlinNoise {
            seed: 1,
            scale: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            octaves: 0,
            persistence: 0.5,
        };
        let v = noise.sample(0.25);
        assert!(v.is_finite());
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn test_perlin_output_range() {
        let noise = PerlinNoise::default();
        let range = noise.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);

        let noise2 = PerlinNoise::with_offset(42, 1.0, 0.5, 0.25);
        let range2 = noise2.output_range();
        assert!((range2.min - (-0.25)).abs() < 0.001);
        assert!((range2.max - 0.75).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/noise/cls_perlin.rs</FILE> - <DESC>Perlin-like smooth noise generator</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
