// <FILE>src/core/noise_helpers.rs</FILE> - <DESC>Common noise generator helpers</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Noise helpers refactor - extracting common patterns</WCTX>
// <CLOG>Initial creation with 7 helper functions</CLOG>

use crate::math::{derive_seed, finite_or, finite_or_f64};
use crate::traits::{SignalContext, SignalRange, SignalTime};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// --- Pattern #1: Bipolar Output Range ---

/// Compute SignalRange for bipolar [-amplitude, +amplitude] centered at offset.
///
/// Handles NaN/Inf inputs by falling back to defaults (amplitude=1.0, offset=0.0).
///
/// # Example
/// ```
/// use mixed_signals::core::bipolar_range;
/// let range = bipolar_range(0.5, 0.25);
/// assert!((range.min - (-0.25)).abs() < 0.001);
/// assert!((range.max - 0.75).abs() < 0.001);
/// ```
#[inline]
pub fn bipolar_range(amplitude: f32, offset: f32) -> SignalRange {
    let amplitude = finite_or(amplitude, 1.0);
    let offset = finite_or(offset, 0.0);
    SignalRange::new(offset - amplitude, offset + amplitude)
}

// --- Pattern #2: RNG from Time ---

/// Create deterministic ChaCha8Rng from seed and time.
///
/// Converts time to milliseconds for seed derivation, ensuring
/// consistent random sequences for the same (seed, time) pair.
///
/// # Example
/// ```
/// use mixed_signals::core::rng_from_time;
/// use rand::RngCore;
/// let mut rng = rng_from_time(42, 1.5);
/// let value = rng.next_u64();
/// ```
#[inline]
pub fn rng_from_time(seed: u64, t: SignalTime) -> ChaCha8Rng {
    let time_ms = (finite_or_f64(t, 0.0) * 1000.0) as u64;
    let seed_bytes = derive_seed(seed, time_ms);
    ChaCha8Rng::from_seed(seed_bytes)
}

// --- Pattern #3: RNG from Context ---

/// Create deterministic ChaCha8Rng from base seed and SignalContext.
///
/// Combines base seed with context seed, and time with frame number
/// for maximum entropy while maintaining determinism.
///
/// # Example
/// ```
/// use mixed_signals::core::rng_from_context;
/// use mixed_signals::traits::SignalContext;
/// use rand::RngCore;
/// let ctx = SignalContext::new(100, 42);
/// let mut rng = rng_from_context(12345, 0.5, &ctx);
/// let value = rng.next_u64();
/// ```
#[inline]
pub fn rng_from_context(base_seed: u64, t: SignalTime, ctx: &SignalContext) -> ChaCha8Rng {
    let effective_seed = base_seed.wrapping_add(ctx.seed);
    let time_ms = (finite_or_f64(t, 0.0) * 1000.0) as u64;
    let combined_input = time_ms.wrapping_add(ctx.frame);
    let seed_bytes = derive_seed(effective_seed, combined_input);
    ChaCha8Rng::from_seed(seed_bytes)
}

// --- Pattern #4: u64 to Bipolar ---

/// Convert u64 random value to bipolar [-1.0, 1.0] range.
///
/// Maps the full u64 range [0, u64::MAX] to [-1.0, 1.0].
///
/// # Example
/// ```
/// use mixed_signals::core::u64_to_bipolar;
/// assert!((u64_to_bipolar(0) - (-1.0)).abs() < 0.001);
/// assert!((u64_to_bipolar(u64::MAX) - 1.0).abs() < 0.001);
/// ```
#[inline]
pub fn u64_to_bipolar(value: u64) -> f64 {
    (value as f64 / u64::MAX as f64) * 2.0 - 1.0
}

// --- Pattern #5: Bipolar Scaling ---

/// Scale bipolar value by amplitude and offset.
///
/// Handles NaN/Inf inputs by falling back to defaults.
///
/// # Example
/// ```
/// use mixed_signals::core::scale_bipolar;
/// assert!((scale_bipolar(0.0, 1.0, 0.0) - 0.0).abs() < 0.001);
/// assert!((scale_bipolar(1.0, 0.5, 0.25) - 0.75).abs() < 0.001);
/// ```
#[inline]
pub fn scale_bipolar(bipolar: f64, amplitude: f32, offset: f32) -> f32 {
    let amplitude = finite_or(amplitude, 1.0);
    let offset = finite_or(offset, 0.0);
    offset + (bipolar as f32) * amplitude
}

// --- Pattern #6: EMA Window Smoothing ---

/// Exponential moving average over lookback window.
///
/// Returns smoothed value using weighted average with correlation decay.
/// Higher correlation means more weight on recent values.
///
/// # Arguments
/// * `frame` - Current frame number
/// * `correlation` - Decay factor (0.0-1.0), higher = smoother
/// * `window` - Number of frames to look back
/// * `sample_fn` - Function that returns bipolar value for a given frame
///
/// # Example
/// ```
/// use mixed_signals::core::ema_smoothing;
/// let result = ema_smoothing(10, 0.9, 5, |_| 0.5);
/// assert!((result - 0.5).abs() < 0.01);
/// ```
#[inline]
pub fn ema_smoothing<F>(frame: u64, correlation: f32, window: u64, sample_fn: F) -> f32
where
    F: Fn(u64) -> f32,
{
    let correlation = finite_or(correlation, 0.95);
    let mut smoothed = 0.0f64;
    let mut weight_sum = 0.0f64;

    for i in 0..window {
        if frame >= i {
            let past_frame = frame - i;
            let value = sample_fn(past_frame) as f64;
            let weight = (correlation as f64).powi(i as i32);
            smoothed += value * weight;
            weight_sum += weight;
        }
    }

    if weight_sum > 0.0 {
        (smoothed / weight_sum) as f32
    } else {
        0.0
    }
}

// --- Pattern #7: Octave Summation ---

/// Sum multiple octaves with 1/f amplitude falloff for pink noise.
///
/// Returns normalized sum in bipolar range. Each octave samples at
/// half the frequency of the previous (frame >> octave).
///
/// # Arguments
/// * `seed` - Base seed for RNG
/// * `frame` - Current frame number
/// * `num_octaves` - Number of octaves to sum (typically 5)
/// * `sample_fn` - Function (octave_seed, octave_frame) -> bipolar value
///
/// # Example
/// ```
/// use mixed_signals::core::octave_sum;
/// let result = octave_sum(42, 100, 5, |_, _| 0.5);
/// assert!((result - 0.5).abs() < 0.01);
/// ```
#[inline]
pub fn octave_sum<F>(seed: u64, frame: u64, num_octaves: usize, sample_fn: F) -> f32
where
    F: Fn(u64, u64) -> f32,
{
    let mut sum = 0.0f64;
    let mut normalizer = 0.0f64;

    for octave in 0..num_octaves {
        let octave_seed = seed.wrapping_add(octave as u64 * 1000);
        let octave_frame = frame >> octave;
        let value = sample_fn(octave_seed, octave_frame) as f64;
        let weight = 1.0 / (octave as f64 + 1.0);
        sum += value * weight;
        normalizer += weight;
    }

    if normalizer > 0.0 {
        (sum / normalizer) as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bipolar_range_default() {
        let range = bipolar_range(1.0, 0.0);
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_bipolar_range_with_offset() {
        let range = bipolar_range(0.5, 0.25);
        assert!((range.min - (-0.25)).abs() < 0.001);
        assert!((range.max - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_bipolar_range_nan_fallback() {
        let range = bipolar_range(f32::NAN, f32::NAN);
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_u64_to_bipolar_edges() {
        assert!((u64_to_bipolar(0) - (-1.0)).abs() < 0.001);
        assert!((u64_to_bipolar(u64::MAX) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_u64_to_bipolar_midpoint() {
        assert!((u64_to_bipolar(u64::MAX / 2) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_bipolar_identity() {
        assert!((scale_bipolar(0.0, 1.0, 0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_scale_bipolar_with_params() {
        assert!((scale_bipolar(1.0, 0.5, 0.25) - 0.75).abs() < 0.001);
        assert!((scale_bipolar(-1.0, 0.5, 0.25) - (-0.25)).abs() < 0.001);
    }

    #[test]
    fn test_scale_bipolar_nan_fallback() {
        let result = scale_bipolar(0.5, f32::NAN, f32::NAN);
        assert!((result - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_rng_from_time_deterministic() {
        use rand::RngCore;
        let mut rng1 = rng_from_time(42, 1.5);
        let mut rng2 = rng_from_time(42, 1.5);
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_rng_from_time_different_seeds() {
        use rand::RngCore;
        let mut rng1 = rng_from_time(42, 1.5);
        let mut rng2 = rng_from_time(43, 1.5);
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_rng_from_time_different_times() {
        use rand::RngCore;
        let mut rng1 = rng_from_time(42, 1.5);
        let mut rng2 = rng_from_time(42, 2.5);
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_rng_from_context_deterministic() {
        use rand::RngCore;
        let ctx = SignalContext::new(100, 42);
        let mut rng1 = rng_from_context(12345, 0.5, &ctx);
        let mut rng2 = rng_from_context(12345, 0.5, &ctx);
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_rng_from_context_different_frames() {
        use rand::RngCore;
        let ctx1 = SignalContext::new(100, 42);
        let ctx2 = SignalContext::new(101, 42);
        let mut rng1 = rng_from_context(12345, 0.5, &ctx1);
        let mut rng2 = rng_from_context(12345, 0.5, &ctx2);
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_ema_smoothing_constant() {
        let result = ema_smoothing(10, 0.9, 5, |_| 0.5);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ema_smoothing_zero_frame() {
        let result = ema_smoothing(0, 0.9, 5, |_| 0.5);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_octave_sum_constant() {
        let result = octave_sum(42, 100, 5, |_, _| 0.5);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_octave_sum_zero_octaves() {
        let result = octave_sum(42, 100, 0, |_, _| 0.5);
        assert!((result - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_octave_sum_single_octave() {
        let result = octave_sum(42, 100, 1, |_, _| 0.7);
        assert!((result - 0.7).abs() < 0.01);
    }
}

// <FILE>src/core/noise_helpers.rs</FILE> - <DESC>Common noise generator helpers</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
