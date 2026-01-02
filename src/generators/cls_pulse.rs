// <FILE>mixed-signals/src/generators/cls_pulse.rs</FILE> - <DESC>Pulse window signal</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Pulse signal that is high during a specific time window.
///
/// Returns `high` value when start <= t < end, otherwise returns `low`.
/// Useful for triggering effects during specific time ranges.
///
/// Use `.normalized()` if output values exceed [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pulse {
    /// Value outside the pulse window
    pub low: f32,
    /// Value inside the pulse window
    pub high: f32,
    /// Start of pulse window
    pub start: f32,
    /// End of pulse window
    pub end: f32,
}

impl Pulse {
    pub fn new(low: f32, high: f32, start: f32, end: f32) -> Self {
        Self {
            low,
            high,
            start,
            end,
        }
    }

    /// Pulse from 0 to 1 during the given window.
    pub fn window(start: f32, end: f32) -> Self {
        Self::new(0.0, 1.0, start, end)
    }
}

impl Default for Pulse {
    fn default() -> Self {
        Self {
            low: 0.0,
            high: 1.0,
            start: 0.25,
            end: 0.75,
        }
    }
}

impl Signal for Pulse {
    fn output_range(&self) -> SignalRange {
        let low = finite_or(self.low, 0.0);
        let high = finite_or(self.high, 1.0);
        SignalRange::new(low.min(high), low.max(high))
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let low = finite_or(self.low, 0.0);
        let high = finite_or(self.high, 1.0);
        let start = finite_or(self.start, 0.25);
        let end = finite_or(self.end, 0.75);

        if t >= start as f64 && t < end as f64 {
            high
        } else {
            low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_before_window() {
        let pulse = Pulse::default();
        assert!((pulse.sample(0.0) - 0.0).abs() < 0.001);
        assert!((pulse.sample(0.24) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_pulse_inside_window() {
        let pulse = Pulse::default();
        assert!((pulse.sample(0.25) - 1.0).abs() < 0.001);
        assert!((pulse.sample(0.5) - 1.0).abs() < 0.001);
        assert!((pulse.sample(0.74) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pulse_after_window() {
        let pulse = Pulse::default();
        assert!((pulse.sample(0.75) - 0.0).abs() < 0.001);
        assert!((pulse.sample(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_pulse_custom_values() {
        let pulse = Pulse::new(0.2, 0.8, 0.3, 0.4);
        assert!((pulse.sample(0.2) - 0.2).abs() < 0.001);
        assert!((pulse.sample(0.35) - 0.8).abs() < 0.001);
        assert!((pulse.sample(0.5) - 0.2).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/generators/cls_pulse.rs</FILE> - <DESC>Pulse window signal</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-27</VERS>
