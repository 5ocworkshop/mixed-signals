// <FILE>mixed-signals/src/generators/cls_ramp.rs</FILE> - <DESC>Linear ramp signal</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Linear ramp from start value to end value over a duration.
///
/// Commonly used for progress-based animations (0..1 over animation duration).
/// Clamps to end value after duration.
///
/// Use `.normalized()` if output values exceed [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ramp {
    /// Starting value
    pub start: f32,
    /// Ending value
    pub end: f32,
    /// Duration in seconds (or use normalized 0..1 time)
    pub duration: f32,
}

impl Ramp {
    pub fn new(start: f32, end: f32, duration: f32) -> Self {
        Self {
            start,
            end,
            duration: duration.max(0.001), // Avoid division by zero
        }
    }

    /// Ramp from 0 to 1 over the given duration.
    pub fn normalized(duration: f32) -> Self {
        Self::new(0.0, 1.0, duration)
    }

    /// Ramp from 0 to 1 over 1 second (use with normalized progress).
    pub fn unit() -> Self {
        Self::new(0.0, 1.0, 1.0)
    }
}

impl Default for Ramp {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
            duration: 1.0,
        }
    }
}

impl Signal for Ramp {
    fn output_range(&self) -> SignalRange {
        let start = finite_or(self.start, 0.0);
        let end = finite_or(self.end, 1.0);
        SignalRange::new(start.min(end), start.max(end))
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let start = finite_or(self.start, 0.0) as f64;
        let end = finite_or(self.end, 1.0) as f64;
        let duration = if self.duration.is_finite() {
            (self.duration.max(0.001)) as f64
        } else {
            0.001_f64
        };

        let progress = (t / duration).clamp(0.0, 1.0);
        (start + (end - start) * progress) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramp_at_start() {
        let ramp = Ramp::default();
        assert!((ramp.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_ramp_at_end() {
        let ramp = Ramp::default();
        assert!((ramp.sample(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ramp_at_half() {
        let ramp = Ramp::default();
        assert!((ramp.sample(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_ramp_clamps_past_end() {
        let ramp = Ramp::default();
        assert!((ramp.sample(2.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ramp_custom_range() {
        let ramp = Ramp::new(0.2, 0.8, 2.0);
        assert!((ramp.sample(0.0) - 0.2).abs() < 0.001);
        assert!((ramp.sample(1.0) - 0.5).abs() < 0.001);
        assert!((ramp.sample(2.0) - 0.8).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/generators/cls_ramp.rs</FILE> - <DESC>Linear ramp signal</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-27</VERS>
