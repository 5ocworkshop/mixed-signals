// <FILE>mixed-signals/src/processing/cls_abs.rs</FILE> - <DESC>Signal absolute value operator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor</WCTX>
// <CLOG>Removed clamping, added output_range() for bipolar compatibility</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Takes the absolute value of a signal.
///
/// Output = |signal|
///
/// Useful for rectifying oscillators (e.g., full-wave rectification of sine).
/// For a bipolar sine [-1, 1], Abs produces [0, 1].
#[derive(Debug, Clone)]
pub struct Abs<S> {
    pub signal: S,
}

impl<S: Signal> Abs<S> {
    pub fn new(signal: S) -> Self {
        Self { signal }
    }
}

impl<S: Signal> Signal for Abs<S> {
    fn output_range(&self) -> SignalRange {
        let r = self.signal.output_range();
        // Abs of range: max is max(|min|, |max|)
        // min is 0 if range spans zero, otherwise min(|min|, |max|)
        let abs_min = r.min.abs();
        let abs_max = r.max.abs();
        let new_max = abs_min.max(abs_max);
        let new_min = if r.min <= 0.0 && r.max >= 0.0 {
            0.0
        } else {
            abs_min.min(abs_max)
        };
        SignalRange::new(new_min, new_max)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.signal.sample(t).abs()
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        self.signal.sample_with_context(t, ctx).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    struct BipolarConstant(f32);

    impl Signal for BipolarConstant {
        fn output_range(&self) -> SignalRange {
            SignalRange::BIPOLAR
        }
        fn sample(&self, _t: SignalTime) -> f32 {
            self.0
        }
    }

    #[test]
    fn test_abs_positive() {
        let sig = Constant::new(0.5);
        let abs_sig = Abs::new(sig);
        assert!((abs_sig.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_abs_negative() {
        let sig = BipolarConstant(-0.5);
        let abs_sig = Abs::new(sig);
        assert!((abs_sig.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_abs_sine() {
        // Abs of bipolar sine rectifies it to [0, 1]
        let sine = Sine::with_frequency(1.0);
        let abs_sine = Abs::new(sine);

        // At t=0, sin=0, abs=0
        assert!((abs_sine.sample(0.0) - 0.0).abs() < 0.001);
        // At t=0.25, sin=1, abs=1
        assert!((abs_sine.sample(0.25) - 1.0).abs() < 0.001);
        // At t=0.75, sin=-1, abs=1
        assert!((abs_sine.sample(0.75) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_abs_output_range() {
        // Bipolar [-1, 1] abs'd becomes [0, 1]
        let sine = Sine::with_frequency(1.0);
        let abs_sine = Abs::new(sine);
        let range = abs_sine.output_range();
        assert!((range.min - 0.0).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);

        // Positive-only range [0.2, 0.8] stays [0.2, 0.8]
        struct PositiveSignal;
        impl Signal for PositiveSignal {
            fn output_range(&self) -> SignalRange {
                SignalRange::new(0.2, 0.8)
            }
            fn sample(&self, _t: SignalTime) -> f32 {
                0.5
            }
        }
        let abs_pos = Abs::new(PositiveSignal);
        let range2 = abs_pos.output_range();
        assert!((range2.min - 0.2).abs() < 0.001);
        assert!((range2.max - 0.8).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/processing/cls_abs.rs</FILE> - <DESC>Signal absolute value operator</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
