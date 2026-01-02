// <FILE>src/random/cls_impulse_noise.rs</FILE> - <DESC>Impulse/shot noise generator</DESC>
// <VERS>VERSION: 2.1.1</VERS>
// <WCTX>Clippy fixes</WCTX>
// <CLOG>Use clamp() instead of max().min() pattern</CLOG>

use crate::core::bipolar_range;
use crate::math::{derive_seed, finite_or, finite_or_f64};
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Exp};
use serde::{Deserialize, Serialize};

/// Impulse/shot noise generator.
///
/// Generates discrete impulse events based on a Poisson process. Unlike
/// `PoissonNoise` which produces continuous value noise, `ImpulseNoise`
/// generates actual discrete spikes suitable for percussive sounds,
/// trigger signals, and sparse texture generation.
///
/// # Output (Bipolar)
///
/// - Returns +amplitude (default +1.0) during an impulse
/// - Returns -amplitude (default -1.0) otherwise
/// - Use `.normalized()` to convert to [0, 1] range for gating
///
/// # Stateless Implementation
///
/// To maintain statelesness (pure `sample(t)`), the timeline is divided into
/// buckets. Each bucket deterministically decides whether it contains an
/// impulse based on the seed and bucket index.
///
/// # Examples
///
/// ```
/// use mixed_signals::random::ImpulseNoise;
/// use mixed_signals::traits::Signal;
///
/// // 10 impulses per second on average
/// let impulses = ImpulseNoise::new(10.0, 42);
/// let v = impulses.sample(0.0);
/// assert!(v == -1.0 || v == 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImpulseNoise {
    /// Average number of impulses per second
    rate_hz: f32,
    /// Seed for reproducible impulse timing
    seed: u64,
    /// Duration of each impulse in seconds
    impulse_width: f32,
    /// Size of time buckets for stateless implementation
    bucket_size: f32,
    /// Output amplitude (scales the output)
    amplitude: f32,
    /// Center value (shifts the output)
    offset: f32,
}

impl ImpulseNoise {
    /// Create a new impulse noise generator.
    ///
    /// # Arguments
    /// * `rate_hz` - Average impulses per second (Poisson λ parameter)
    /// * `seed` - Seed for deterministic impulse timing
    pub fn new(rate_hz: f32, seed: u64) -> Self {
        Self {
            rate_hz: rate_hz.max(0.0),
            seed,
            impulse_width: 0.001, // 1ms default
            bucket_size: 0.1,     // 100ms buckets
            amplitude: 1.0,
            offset: 0.0,
        }
    }

    /// Create with custom impulse width.
    ///
    /// # Arguments
    /// * `rate_hz` - Average impulses per second
    /// * `seed` - Seed for deterministic timing
    /// * `impulse_width` - Duration of each impulse in seconds
    pub fn with_width(rate_hz: f32, seed: u64, impulse_width: f32) -> Self {
        Self {
            rate_hz: rate_hz.max(0.0),
            seed,
            impulse_width: impulse_width.max(0.0001), // Minimum 0.1ms
            bucket_size: 0.1,
            amplitude: 1.0,
            offset: 0.0,
        }
    }

    /// Create with custom bucket size for different accuracy/performance tradeoff.
    pub fn with_bucket_size(rate_hz: f32, seed: u64, bucket_size: f32) -> Self {
        Self {
            rate_hz: rate_hz.max(0.0),
            seed,
            impulse_width: 0.001,
            bucket_size: bucket_size.clamp(0.01, 1.0), // 10ms to 1s
            amplitude: 1.0,
            offset: 0.0,
        }
    }

    pub fn rate_hz(&self) -> f32 {
        self.rate_hz
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn impulse_width(&self) -> f32 {
        self.impulse_width
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// Check if time t is within an impulse.
    fn is_in_impulse(&self, t: f64, seed: u64) -> bool {
        let rate = finite_or(self.rate_hz, 0.0) as f64;
        if rate <= 0.0 {
            return false;
        }

        let bucket_size = self.bucket_size as f64;
        let impulse_width = self.impulse_width as f64;

        // Determine which bucket this time falls into
        let bucket_index = (t / bucket_size).floor() as i64;
        let bucket_start = bucket_index as f64 * bucket_size;

        // Expected impulses per bucket
        let expected_impulses = rate * bucket_size;

        // Use deterministic RNG for this bucket
        let seed_bytes = derive_seed(seed, bucket_index as u64);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);

        // Decide how many impulses in this bucket (simplified: 0 or 1 for sparse rates)
        // For higher rates, we sample multiple times
        let mut impulse_time = bucket_start;

        // Sample from exponential distribution for inter-arrival times
        let exp_dist = match Exp::new(rate) {
            Ok(dist) => dist,
            Err(_) => return false,
        };

        // Walk through potential impulses in this bucket
        let max_checks = ((expected_impulses * 3.0).ceil() as usize).clamp(1, 10);
        for _ in 0..max_checks {
            let inter_arrival = exp_dist.sample(&mut rng);
            impulse_time += inter_arrival;

            if impulse_time > bucket_start + bucket_size {
                break; // Past this bucket
            }

            // Check if t is within this impulse
            if t >= impulse_time && t < impulse_time + impulse_width {
                return true;
            }
        }

        // Also check impulses from the previous bucket that might extend into this time
        if bucket_index > 0 {
            let prev_bucket_index = bucket_index - 1;
            let prev_bucket_start = prev_bucket_index as f64 * bucket_size;

            let seed_bytes = derive_seed(seed, prev_bucket_index as u64);
            let mut rng = ChaCha8Rng::from_seed(seed_bytes);

            let mut impulse_time = prev_bucket_start;
            for _ in 0..max_checks {
                let inter_arrival = exp_dist.sample(&mut rng);
                impulse_time += inter_arrival;

                if impulse_time > prev_bucket_start + bucket_size {
                    break;
                }

                // Check if this impulse extends into our time
                if t >= impulse_time && t < impulse_time + impulse_width {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for ImpulseNoise {
    fn default() -> Self {
        Self {
            rate_hz: 1.0,
            seed: 0,
            impulse_width: 0.001,
            bucket_size: 0.1,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Signal for ImpulseNoise {
    fn output_range(&self) -> SignalRange {
        bipolar_range(self.amplitude, self.offset)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if t < 0.0 {
            return offset - amplitude; // Low state for negative time
        }

        if self.is_in_impulse(t, self.seed) {
            offset + amplitude // High state (impulse)
        } else {
            offset - amplitude // Low state (no impulse)
        }
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);

        if t < 0.0 {
            return offset - amplitude; // Low state for negative time
        }

        let effective_seed = self.seed.wrapping_add(ctx.seed);
        if self.is_in_impulse(t, effective_seed) {
            offset + amplitude // High state (impulse)
        } else {
            offset - amplitude // Low state (no impulse)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impulse_noise_zero_rate() {
        // Zero rate means always low state (-1.0)
        let impulses = ImpulseNoise::new(0.0, 42);
        for i in 0..100 {
            let t = i as f64 * 0.01;
            assert_eq!(
                impulses.sample(t),
                -1.0,
                "Zero rate should produce no impulses (low state)"
            );
        }
    }

    #[test]
    fn test_impulse_noise_determinism() {
        let impulses = ImpulseNoise::new(10.0, 42);
        let v1 = impulses.sample(0.5);
        let v2 = impulses.sample(0.5);
        assert_eq!(v1, v2, "Same time should produce same value");
    }

    #[test]
    fn test_impulse_noise_different_seeds() {
        let impulses1 = ImpulseNoise::new(100.0, 1);
        let impulses2 = ImpulseNoise::new(100.0, 2);

        // Sample many times and check that at least one differs
        let mut found_difference = false;
        for i in 0..100 {
            let t = i as f64 * 0.01;
            if impulses1.sample(t) != impulses2.sample(t) {
                found_difference = true;
                break;
            }
        }
        assert!(
            found_difference,
            "Different seeds should produce different patterns"
        );
    }

    #[test]
    fn test_impulse_noise_bipolar_output() {
        // Output should be -1 or +1 (bipolar)
        let impulses = ImpulseNoise::new(50.0, 42);
        for i in 0..1000 {
            let t = i as f64 * 0.001;
            let v = impulses.sample(t);
            assert!(
                v == -1.0 || v == 1.0,
                "Output should be bipolar (-1 or 1), got {}",
                v
            );
        }
    }

    #[test]
    fn test_impulse_noise_output_range() {
        let impulses = ImpulseNoise::default();
        let range = impulses.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_impulse_noise_rate_affects_density() {
        let low_rate = ImpulseNoise::new(1.0, 42);
        let high_rate = ImpulseNoise::new(100.0, 42);

        let mut low_count = 0;
        let mut high_count = 0;

        for i in 0..10000 {
            let t = i as f64 * 0.0001; // Sample every 0.1ms over 1 second
            if low_rate.sample(t) > 0.0 {
                low_count += 1;
            }
            if high_rate.sample(t) > 0.0 {
                high_count += 1;
            }
        }

        assert!(
            high_count > low_count,
            "Higher rate {} should produce more impulses than lower rate {}",
            high_count,
            low_count
        );
    }

    #[test]
    fn test_impulse_noise_negative_rate() {
        let impulses = ImpulseNoise::new(-5.0, 42);
        // Should treat as zero rate, always low state
        for i in 0..100 {
            let t = i as f64 * 0.01;
            assert_eq!(impulses.sample(t), -1.0);
        }
    }

    #[test]
    fn test_impulse_noise_with_width() {
        let impulses = ImpulseNoise::with_width(10.0, 42, 0.01);
        assert_eq!(impulses.impulse_width(), 0.01);
    }

    #[test]
    fn test_impulse_noise_with_context() {
        let impulses = ImpulseNoise::new(50.0, 42);
        let ctx = SignalContext::new(100, 999);
        let v = impulses.sample_with_context(0.5, &ctx);
        assert!(v == -1.0 || v == 1.0);
    }

    #[test]
    fn test_impulse_noise_negative_time() {
        let impulses = ImpulseNoise::new(10.0, 42);
        // Negative time returns low state
        assert_eq!(impulses.sample(-1.0), -1.0);
    }
}

// <FILE>src/random/cls_impulse_noise.rs</FILE> - <DESC>Impulse/shot noise generator</DESC>
// <VERS>END OF VERSION: 2.1.1</VERS>
