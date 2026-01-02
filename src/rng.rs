// <FILE>src/rng.rs</FILE> - <DESC>Unified RNG API for common randomness patterns</DESC>
// <VERS>VERSION: 1.4.0</VERS>
// <WCTX>Migrate time types from f32 to f64</WCTX>
// <CLOG>Internal time changed to f64 to match Signal trait</CLOG>

//! Central RNG interface for common randomness needs.
//!
//! This module provides a unified API for generating random values, wrapping
//! the signal-based random generators for ease of use. Use this when you need
//! simple random values; use the underlying signals when you need time-based
//! or context-aware randomness.
//!
//! # Quick Start
//!
//! ```rust
//! use mixed_signals::rng::Rng;
//!
//! let mut rng = Rng::with_seed(42);
//!
//! // Uniform random in range
//! let value = rng.uniform(0.0, 10.0);
//!
//! // Gaussian (normal) distribution
//! let gaussian = rng.gaussian(0.0, 1.0).unwrap();
//!
//! // Discrete events (Poisson)
//! let events = rng.poisson(2.5).unwrap();
//! ```
//!
//! # When to use Rng vs Signals
//!
//! **Use `Rng`:**
//! - One-off random values
//! - Simple uniform, gaussian, or poisson randomness
//! - Traditional RNG usage patterns
//!
//! `Rng::gaussian` and `Rng::poisson` return distribution values without unit clamping.
//!
//! **Use signals directly (WhiteNoise, GaussianNoise, etc):**
//! - Time-based randomness (varying over t)
//! - Context-aware randomness (frame, phase, spatial)
//! - Composable randomness (mix with other signals)
//! - Serializable random configurations

use crate::math::derive_seed;
use crate::random::SeededRandom;
use crate::traits::Signal;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal, Poisson};

/// Central RNG for common randomness patterns.
///
/// Wraps signal-based random generators for traditional RNG usage.
/// Each call advances an internal time counter for deterministic sequences.
pub struct Rng {
    seed: u64,
    time: crate::traits::SignalTime,
    time_step: crate::traits::SignalTime,
}

impl Rng {
    /// Create a new RNG with a specific seed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::new(42);
    /// let value = rng.uniform(0.0, 1.0);
    /// ```
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            time: 0.0,
            time_step: 0.001, // Small step for fine-grained randomness
        }
    }

    /// Alias for `new()` - create a new RNG with a specific seed.
    ///
    /// Provided for consistency with other types that use `with_seed()`.
    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed)
    }

    /// Create a new RNG from system entropy (non-deterministic).
    ///
    /// **Warning:** Not reproducible across runs. Use `with_seed()` for determinism.
    #[cfg(feature = "std")]
    pub fn from_entropy() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self::with_seed(seed)
    }

    /// Generate a uniform random value in the range [min, max].
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let dice_roll = rng.uniform(1.0, 7.0).floor() as i32; // 1-6
    /// ```
    pub fn uniform(&mut self, min: f32, max: f32) -> f32 {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let noise = SeededRandom::with_seed(self.seed);
        let value = noise.sample(self.time);
        self.time += self.time_step;

        // Map from [0, 1] to [min, max]
        min + value * (max - min)
    }

    /// Generate a value from a Gaussian (normal) distribution.
    ///
    /// # Arguments
    ///
    /// * `mean` - Center of the distribution
    /// * `std_dev` - Standard deviation (spread)
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let height = rng.gaussian(170.0, 10.0).unwrap(); // Mean 170cm, stddev 10cm
    /// ```
    pub fn gaussian(&mut self, mean: f32, std_dev: f32) -> Result<f32, String> {
        if std_dev < 0.0 || !std_dev.is_finite() || !mean.is_finite() {
            return Err(format!(
                "Gaussian std_dev must be >= 0 and finite, got {}",
                std_dev
            ));
        }

        if std_dev == 0.0 {
            self.time += self.time_step;
            return Ok(mean);
        }

        let time_ms = (self.time * 1000.0) as u64;
        let seed_bytes = derive_seed(self.seed, time_ms);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let normal = Normal::new(mean as f64, std_dev as f64)
            .map_err(|_| format!("Gaussian std_dev must be > 0, got {}", std_dev))?;
        let value = normal.sample(&mut rng) as f32;
        self.time += self.time_step;
        Ok(value)
    }

    /// Generate a value from a Poisson distribution.
    ///
    /// Useful for modeling discrete random events (network packets, user clicks, etc).
    /// Returns a sampled count as f32 (mean ~ lambda).
    ///
    /// # Arguments
    ///
    /// * `lambda` - Average event rate (must be > 0)
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let packet_count = rng.poisson(5.0).unwrap(); // Average 5 packets per interval
    /// ```
    pub fn poisson(&mut self, lambda: f32) -> Result<f32, String> {
        if lambda <= 0.0 || !lambda.is_finite() {
            return Err(format!(
                "Poisson lambda must be > 0 and finite, got {}",
                lambda
            ));
        }

        let time_ms = (self.time * 1000.0) as u64;
        let seed_bytes = derive_seed(self.seed, time_ms);
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let poisson = Poisson::new(lambda as f64)
            .map_err(|_| format!("Poisson lambda must be > 0, got {}", lambda))?;
        let value = poisson.sample(&mut rng) as f32;
        self.time += self.time_step;
        Ok(value)
    }

    /// Generate a boolean with given probability of being true.
    ///
    /// # Arguments
    ///
    /// * `probability` - Probability of true (0.0 to 1.0)
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// if rng.chance(0.7) {
    ///     // 70% chance of this happening
    /// }
    /// ```
    pub fn chance(&mut self, probability: f32) -> bool {
        if probability.is_nan() {
            return false;
        }
        if probability <= 0.0 {
            return false;
        }
        if probability >= 1.0 {
            return true;
        }
        self.uniform(0.0, 1.0) < probability
    }

    /// Choose a random element from a slice.
    ///
    /// Returns `None` if the slice is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let colors = ["red", "green", "blue"];
    /// let chosen = rng.choose(&colors);
    /// ```
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let index = (self.uniform(0.0, items.len() as f32).floor() as usize).min(items.len() - 1);
        Some(&items[index])
    }

    /// Shuffle a slice in place using Fisher-Yates algorithm.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let mut deck = vec![1, 2, 3, 4, 5];
    /// rng.shuffle(&mut deck);
    /// ```
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.uniform(0.0, (i + 1) as f32).floor() as usize).min(i);
            items.swap(i, j);
        }
    }

    /// Partially shuffle a slice, randomizing only the first k elements.
    ///
    /// More efficient than full shuffle when you only need a subset.
    /// After calling, `items[0..k]` contains k uniformly random elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let mut deck: Vec<i32> = (1..=52).collect();
    /// rng.shuffle_partial(&mut deck, 5);
    /// let hand: Vec<i32> = deck[..5].to_vec(); // 5 random cards
    /// ```
    pub fn shuffle_partial<T>(&mut self, items: &mut [T], k: usize) {
        let len = items.len();
        if len <= 1 || k == 0 {
            return;
        }
        let k = k.min(len);
        for i in 0..k {
            let j = (self.uniform(0.0, (len - i) as f32).floor() as usize).min(len - i - 1) + i;
            items.swap(i, j);
        }
    }

    /// Cyclic shuffle - no element stays in its original position.
    ///
    /// Uses Sattolo's algorithm to generate a cyclic permutation (derangement).
    /// Useful for Secret Santa assignments or when every element must move.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let original = vec!["Alice", "Bob", "Carol", "Dave"];
    /// let mut assignments = original.clone();
    /// rng.shuffle_cyclic(&mut assignments);
    ///
    /// // No one has their original position
    /// for (i, &name) in assignments.iter().enumerate() {
    ///     assert_ne!(name, original[i]);
    /// }
    /// ```
    pub fn shuffle_cyclic<T>(&mut self, items: &mut [T]) {
        let len = items.len();
        if len <= 1 {
            return;
        }
        for i in (1..len).rev() {
            let j = (self.uniform(0.0, i as f32).floor() as usize).min(i - 1);
            items.swap(i, j);
        }
    }

    /// Weighted shuffle - higher weight means earlier position.
    ///
    /// Uses the Efraimidis-Spirakis algorithm for unbiased weighted sampling.
    /// Ideal for loot tables, priority queues, and sponsored content mixing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::rng::Rng;
    ///
    /// let mut rng = Rng::with_seed(42);
    /// let mut items = vec!["common", "rare", "legendary"];
    /// let weights = [10.0, 3.0, 1.0];  // common appears first more often
    /// rng.shuffle_weighted(&mut items, &weights);
    /// ```
    pub fn shuffle_weighted<T>(&mut self, items: &mut [T], weights: &[f32]) {
        crate::shuffle::weighted_shuffle(items, weights, self);
    }

    /// Reset the internal time counter.
    ///
    /// Useful for restarting a deterministic sequence.
    pub fn reset(&mut self) {
        self.time = 0.0;
    }

    /// Get the current seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Change the seed, resetting the sequence.
    pub fn reseed(&mut self, seed: u64) {
        self.seed = seed;
        self.time = 0.0;
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::with_seed(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = Rng::with_seed(42);
        let mut rng2 = Rng::with_seed(42);

        for _ in 0..10 {
            assert_eq!(rng1.uniform(0.0, 1.0), rng2.uniform(0.0, 1.0));
        }
    }

    #[test]
    fn test_rng_uniform_range() {
        let mut rng = Rng::with_seed(42);

        for _ in 0..100 {
            let value = rng.uniform(5.0, 10.0);
            assert!((5.0..=10.0).contains(&value));
        }
    }

    #[test]
    fn test_rng_uniform_swaps_min_max() {
        let mut rng = Rng::with_seed(42);
        let value = rng.uniform(10.0, 5.0);
        assert!((5.0..=10.0).contains(&value));
    }

    #[test]
    fn test_rng_gaussian() {
        let mut rng = Rng::with_seed(42);

        // Generate values and check they're finite
        for _ in 0..100 {
            let value = rng.gaussian(0.0, 1.0).unwrap();
            assert!(value.is_finite());
        }
    }

    #[test]
    fn test_rng_gaussian_invalid_returns_err() {
        let mut rng = Rng::with_seed(42);
        assert!(rng.gaussian(0.0, -1.0).is_err());
    }

    #[test]
    fn test_rng_gaussian_zero_std_dev_returns_mean() {
        let mut rng = Rng::with_seed(42);
        let value = rng.gaussian(2.5, 0.0).unwrap();
        assert_eq!(value, 2.5);
    }

    #[test]
    fn test_rng_chance() {
        let mut rng = Rng::with_seed(42);

        let mut true_count = 0;
        for _ in 0..1000 {
            if rng.chance(0.7) {
                true_count += 1;
            }
        }

        // Should be roughly 70% true (allow 10% variance)
        assert!(true_count > 600 && true_count < 800);
    }

    #[test]
    fn test_rng_chance_nan_false() {
        let mut rng = Rng::with_seed(42);
        assert!(!rng.chance(f32::NAN));
    }

    #[test]
    fn test_rng_chance_bounds() {
        let mut rng = Rng::with_seed(42);
        assert!(!rng.chance(-1.0));
        assert!(rng.chance(2.0));
    }

    #[test]
    fn test_rng_choose() {
        let mut rng = Rng::with_seed(42);
        let items = ["a", "b", "c"];

        let chosen = rng.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(chosen.unwrap()));

        // Empty slice returns None
        let empty: &[i32] = &[];
        assert!(rng.choose(empty).is_none());
    }

    #[test]
    fn test_rng_shuffle() {
        let mut rng = Rng::with_seed(42);
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();

        rng.shuffle(&mut items);

        // Should contain same elements
        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, original);

        // Should be in different order (probabilistic, but seed 42 does shuffle)
        assert_ne!(items, original);
    }

    #[test]
    fn test_rng_reset() {
        let mut rng = Rng::with_seed(42);
        let v1 = rng.uniform(0.0, 1.0);
        let v2 = rng.uniform(0.0, 1.0);
        assert_ne!(v1, v2);

        rng.reset();
        let v3 = rng.uniform(0.0, 1.0);
        assert_eq!(v1, v3); // Should match first value after reset
    }

    #[test]
    fn test_rng_reseed() {
        let mut rng = Rng::with_seed(42);
        let v1 = rng.uniform(0.0, 1.0);

        rng.reseed(99);
        let v2 = rng.uniform(0.0, 1.0);
        assert_ne!(v1, v2); // Different seed = different values

        rng.reseed(42);
        let v3 = rng.uniform(0.0, 1.0);
        assert_eq!(v1, v3); // Same seed = same sequence
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_rng_from_entropy_available() {
        let mut rng = Rng::from_entropy();
        let value = rng.uniform(0.0, 1.0);
        assert!(value.is_finite());
    }

    #[test]
    fn test_rng_poisson_invalid_returns_err() {
        let mut rng = Rng::with_seed(42);
        assert!(rng.poisson(0.0).is_err());
    }
}

// <FILE>src/rng.rs</FILE> - <DESC>Unified RNG API for common randomness patterns</DESC>
// <VERS>END OF VERSION: 1.4.0</VERS>
