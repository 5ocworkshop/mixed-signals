// <FILE>mixed-signals/src/processing/cls_invert.rs</FILE> - <DESC>Signal inversion operator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor</WCTX>
// <CLOG>Removed clamping, added output_range() - now negates signal</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Negates a signal.
///
/// Output = -signal
///
/// For bipolar signals [-1, 1], this flips the polarity.
/// For unit signals [0, 1], use Remap(0,1,1,0) instead if you want 1-x behavior.
#[derive(Debug, Clone)]
pub struct Invert<S> {
    pub signal: S,
}

impl<S: Signal> Invert<S> {
    pub fn new(signal: S) -> Self {
        Self { signal }
    }
}

impl<S: Signal> Signal for Invert<S> {
    fn output_range(&self) -> SignalRange {
        let r = self.signal.output_range();
        // Negation swaps and negates bounds
        SignalRange::new(-r.max, -r.min)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        -self.signal.sample(t)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        -self.signal.sample_with_context(t, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_invert_constant() {
        // Constant(0.5) inverted is -0.5
        let sig = Constant::new(0.5);
        let inverted = Invert::new(sig);
        assert!((inverted.sample(0.0) - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_invert_sine() {
        // Bipolar sine at t=0.25 is 1.0, inverted is -1.0
        // Bipolar sine at t=0.75 is -1.0, inverted is 1.0
        let sine = Sine::with_frequency(1.0);
        let inverted = Invert::new(sine);

        assert!((inverted.sample(0.25) - (-1.0)).abs() < 0.001);
        assert!((inverted.sample(0.75) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_double_invert() {
        // Double negation returns to original
        let sig = Constant::new(0.7);
        let double_inverted = Invert::new(Invert::new(sig));
        assert!((double_inverted.sample(0.0) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_invert_output_range() {
        // Bipolar [-1, 1] inverted is still [-1, 1]
        let sine = Sine::with_frequency(1.0);
        let inverted = Invert::new(sine);
        let range = inverted.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);

        // Constant(0.5) has range [0.5, 0.5], inverted is [-0.5, -0.5]
        let const_sig = Constant::new(0.5);
        let inverted_const = Invert::new(const_sig);
        let range2 = inverted_const.output_range();
        assert!((range2.min - (-0.5)).abs() < 0.001);
        assert!((range2.max - (-0.5)).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/processing/cls_invert.rs</FILE> - <DESC>Signal inversion operator</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
