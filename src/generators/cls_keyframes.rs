// <FILE>mixed-signals/src/generators/cls_keyframes.rs</FILE> - <DESC>Keyframe-based signal with interpolation</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Audio synthesis primitives - arbitrary curve definition</WCTX>
// <CLOG>Initial implementation - piecewise linear interpolation between keyframes</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};
use serde::{Deserialize, Serialize};

/// A keyframe defining a value at a specific time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time position (0.0 to 1.0 normalized, or absolute depending on usage)
    pub time: f32,
    /// Value at this time
    pub value: f32,
}

impl Keyframe {
    pub fn new(time: f32, value: f32) -> Self {
        Self { time, value }
    }
}

/// Signal that interpolates between keyframes.
///
/// Provides piecewise linear interpolation between a series of (time, value)
/// pairs. This enables defining arbitrary curves that can't be expressed
/// with simple oscillators.
///
/// # Stateless Design
///
/// Like all Signal implementations, this is a pure function of time.
/// The keyframe data is immutable configuration. Each sample() call
/// finds the surrounding keyframes and interpolates without any state.
///
/// # Time Normalization
///
/// Input time is expected in the range [0.0, 1.0] for normalized curves,
/// or can be used with absolute time values. The keyframe times define
/// the curve's shape.
///
/// # Examples
///
/// ```
/// use mixed_signals::generators::{Keyframes, Keyframe};
/// use mixed_signals::traits::Signal;
///
/// // Define a simple ramp up then down
/// let curve = Keyframes::new(vec![
///     Keyframe::new(0.0, 0.0),
///     Keyframe::new(0.5, 1.0),
///     Keyframe::new(1.0, 0.0),
/// ]);
///
/// assert!((curve.sample(0.0) - 0.0).abs() < 0.001);
/// assert!((curve.sample(0.25) - 0.5).abs() < 0.001);
/// assert!((curve.sample(0.5) - 1.0).abs() < 0.001);
/// assert!((curve.sample(0.75) - 0.5).abs() < 0.001);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframes {
    /// Sorted list of keyframes (by time)
    keyframes: Vec<Keyframe>,
}

impl Keyframes {
    /// Create a new Keyframes signal from a list of keyframes.
    ///
    /// Keyframes will be sorted by time automatically.
    /// At least one keyframe is required.
    pub fn new(mut keyframes: Vec<Keyframe>) -> Self {
        // Sort by time
        keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Ensure at least one keyframe
        if keyframes.is_empty() {
            keyframes.push(Keyframe::new(0.0, 0.0));
        }

        Self { keyframes }
    }

    /// Create from a slice of (time, value) tuples for convenience.
    pub fn from_pairs(pairs: &[(f32, f32)]) -> Self {
        let keyframes = pairs.iter().map(|&(t, v)| Keyframe::new(t, v)).collect();
        Self::new(keyframes)
    }

    /// Get the number of keyframes.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if empty (should never be true after construction).
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Find the value at time t using binary search and linear interpolation.
    fn interpolate(&self, t: f32) -> f32 {
        let kf = &self.keyframes;

        // Handle edge cases
        if kf.len() == 1 {
            return kf[0].value;
        }

        // Before first keyframe
        if t <= kf[0].time {
            return kf[0].value;
        }

        // After last keyframe
        if t >= kf[kf.len() - 1].time {
            return kf[kf.len() - 1].value;
        }

        // Binary search for the interval containing t
        let idx = match kf
            .binary_search_by(|k| k.time.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(i) => return kf[i].value,   // Exact match
            Err(i) => i.saturating_sub(1), // Insert position - 1 gives lower bound
        };

        // Interpolate between keyframes[idx] and keyframes[idx + 1]
        let k0 = &kf[idx];
        let k1 = &kf[idx + 1];

        let dt = k1.time - k0.time;
        if dt.abs() < 1e-10 {
            return k0.value;
        }

        let progress = (t - k0.time) / dt;
        k0.value + (k1.value - k0.value) * progress
    }
}

impl Signal for Keyframes {
    fn sample(&self, t: SignalTime) -> f32 {
        let t = if t.is_finite() { t as f32 } else { 0.0 };
        self.interpolate(t)
    }

    fn sample_with_context(&self, t: SignalTime, _ctx: &SignalContext) -> f32 {
        self.sample(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframes_single() {
        let kf = Keyframes::new(vec![Keyframe::new(0.5, 0.7)]);
        assert!((kf.sample(0.0) - 0.7).abs() < 0.001);
        assert!((kf.sample(0.5) - 0.7).abs() < 0.001);
        assert!((kf.sample(1.0) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_linear_ramp() {
        let kf = Keyframes::from_pairs(&[(0.0, 0.0), (1.0, 1.0)]);
        assert!((kf.sample(0.0) - 0.0).abs() < 0.001);
        assert!((kf.sample(0.25) - 0.25).abs() < 0.001);
        assert!((kf.sample(0.5) - 0.5).abs() < 0.001);
        assert!((kf.sample(0.75) - 0.75).abs() < 0.001);
        assert!((kf.sample(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_triangle() {
        let kf = Keyframes::from_pairs(&[(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)]);
        assert!((kf.sample(0.0) - 0.0).abs() < 0.001);
        assert!((kf.sample(0.25) - 0.5).abs() < 0.001);
        assert!((kf.sample(0.5) - 1.0).abs() < 0.001);
        assert!((kf.sample(0.75) - 0.5).abs() < 0.001);
        assert!((kf.sample(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_step_like() {
        // Quick jump from 0 to 1 at t=0.1
        let kf = Keyframes::from_pairs(&[(0.0, 0.0), (0.1, 1.0), (1.0, 1.0)]);
        assert!((kf.sample(0.0) - 0.0).abs() < 0.001);
        assert!((kf.sample(0.05) - 0.5).abs() < 0.001);
        assert!((kf.sample(0.1) - 1.0).abs() < 0.001);
        assert!((kf.sample(0.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_before_first() {
        let kf = Keyframes::from_pairs(&[(0.2, 0.5), (0.8, 0.8)]);
        assert!((kf.sample(0.0) - 0.5).abs() < 0.001);
        assert!((kf.sample(0.1) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_after_last() {
        let kf = Keyframes::from_pairs(&[(0.2, 0.5), (0.8, 0.8)]);
        assert!((kf.sample(0.9) - 0.8).abs() < 0.001);
        assert!((kf.sample(1.0) - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_exact_match() {
        let kf = Keyframes::from_pairs(&[(0.0, 0.1), (0.5, 0.5), (1.0, 0.9)]);
        assert!((kf.sample(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_unsorted_input() {
        // Should sort automatically
        let kf = Keyframes::from_pairs(&[(1.0, 1.0), (0.0, 0.0), (0.5, 0.5)]);
        assert!((kf.sample(0.25) - 0.25).abs() < 0.001);
        assert!((kf.sample(0.75) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_empty_creates_default() {
        let kf = Keyframes::new(vec![]);
        assert!((kf.sample(0.5) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_nan_time() {
        let kf = Keyframes::from_pairs(&[(0.0, 0.0), (1.0, 1.0)]);
        // NaN should be treated as 0
        assert!((kf.sample(f64::NAN) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_keyframes_frequency_curve() {
        // Simulate V17's frequency curve (normalized to 0-1)
        let kf = Keyframes::from_pairs(&[
            (0.0, 0.0), // 220 Hz -> 0.0
            (0.1, 1.0), // 577 Hz -> 1.0
            (0.5, 0.6), // ~450 Hz -> 0.6
            (1.0, 0.4), // ~372 Hz -> 0.4
        ]);

        // Rapid rise at start
        assert!(kf.sample(0.05) > 0.4);
        // High at 0.1
        assert!((kf.sample(0.1) - 1.0).abs() < 0.001);
        // Descending by 0.5
        assert!(kf.sample(0.5) < kf.sample(0.1));
    }
}

// <FILE>mixed-signals/src/generators/cls_keyframes.rs</FILE> - <DESC>Keyframe-based signal with interpolation</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
