// <FILE>mixed-signals/src/generators/cls_step.rs</FILE> - <DESC>Step function signal</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Step function that transitions from one value to another at a threshold.
///
/// Returns `before` when t < threshold, `after` when t >= threshold.
///
/// Use `.normalized()` if output values exceed [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// Value before the threshold
    pub before: f32,
    /// Value at and after the threshold
    pub after: f32,
    /// Time at which the step occurs
    pub threshold: f32,
}

impl Step {
    pub fn new(before: f32, after: f32, threshold: f32) -> Self {
        Self {
            before,
            after,
            threshold,
        }
    }

    /// Step from 0 to 1 at the given threshold.
    pub fn at(threshold: f32) -> Self {
        Self::new(0.0, 1.0, threshold)
    }
}

impl Default for Step {
    fn default() -> Self {
        Self {
            before: 0.0,
            after: 1.0,
            threshold: 0.5,
        }
    }
}

impl Signal for Step {
    fn output_range(&self) -> SignalRange {
        let before = finite_or(self.before, 0.0);
        let after = finite_or(self.after, 1.0);
        SignalRange::new(before.min(after), before.max(after))
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let before = finite_or(self.before, 0.0);
        let after = finite_or(self.after, 1.0);
        let threshold = finite_or(self.threshold, 0.5);

        if t < threshold as f64 {
            before
        } else {
            after
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_before_threshold() {
        let step = Step::default();
        assert!((step.sample(0.0) - 0.0).abs() < 0.001);
        assert!((step.sample(0.49) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_step_at_threshold() {
        let step = Step::default();
        assert!((step.sample(0.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_step_after_threshold() {
        let step = Step::default();
        assert!((step.sample(0.51) - 1.0).abs() < 0.001);
        assert!((step.sample(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_step_custom_values() {
        let step = Step::new(0.2, 0.8, 0.25);
        assert!((step.sample(0.1) - 0.2).abs() < 0.001);
        assert!((step.sample(0.3) - 0.8).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/generators/cls_step.rs</FILE> - <DESC>Step function signal</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
