// <FILE>mixed-signals/src/processing/cls_clamp.rs</FILE> - <DESC>Signal clamping operator</DESC>
// <VERS>VERSION: 1.0.0 - 2025-12-22</VERS>
// <WCTX>Signal generator implementation</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Clamps a signal's output to a specified range.
#[derive(Debug, Clone)]
pub struct Clamp<S> {
    pub signal: S,
    pub min: f32,
    pub max: f32,
}

impl<S: Signal> Clamp<S> {
    pub fn new(signal: S, min: f32, max: f32) -> Self {
        let (min, max) = if !min.is_finite() || !max.is_finite() {
            (0.0, 1.0)
        } else if min <= max {
            (min, max)
        } else {
            (max, min)
        };
        Self { signal, min, max }
    }

    /// Clamp to 0..1 range
    pub fn unit(signal: S) -> Self {
        Self::new(signal, 0.0, 1.0)
    }

    /// Clamp to -1..1 range
    pub fn normalized(signal: S) -> Self {
        Self::new(signal, -1.0, 1.0)
    }
}

impl<S: Signal> Signal for Clamp<S> {
    fn output_range(&self) -> SignalRange {
        SignalRange::new(self.min, self.max)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.signal.sample(t).clamp(self.min, self.max)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        self.signal
            .sample_with_context(t, ctx)
            .clamp(self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    #[test]
    fn test_clamp_within_range() {
        let sig = Constant::new(0.5);
        let clamped = Clamp::new(sig, 0.0, 1.0);
        assert!((clamped.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_clamp_above_max() {
        let sig = Constant::new(2.0);
        let clamped = Clamp::new(sig, 0.0, 1.0);
        assert!((clamped.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_below_min() {
        let sig = Constant::new(-2.0);
        let clamped = Clamp::new(sig, 0.0, 1.0);
        assert!((clamped.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_unit() {
        let sig = Constant::new(1.5);
        let clamped = Clamp::unit(sig);
        assert!((clamped.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_swaps_min_max() {
        let sig = Constant::new(0.5);
        let clamped = Clamp::new(sig, 1.0, -1.0);
        assert!((clamped.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_clamp_non_finite_defaults_unit() {
        let sig = Constant::new(0.5);
        let clamped = Clamp::new(sig, f32::NAN, 1.0);
        assert!((clamped.sample(0.0) - 0.5).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/processing/cls_clamp.rs</FILE> - <DESC>Signal clamping operator</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
