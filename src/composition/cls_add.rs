// <FILE>mixed-signals/src/composition/cls_add.rs</FILE> - <DESC>Signal addition operator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Adds two signals together.
///
/// Output = signal_a + signal_b (no clamping)
///
/// Use `.normalized()` to clamp result to [0, 1].
#[derive(Debug, Clone)]
pub struct Add<A, B> {
    pub a: A,
    pub b: B,
}

impl<A: Signal, B: Signal> Add<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Signal, B: Signal> Signal for Add<A, B> {
    fn output_range(&self) -> SignalRange {
        let ra = self.a.output_range();
        let rb = self.b.output_range();
        SignalRange::new(ra.min + rb.min, ra.max + rb.max)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.a.sample(t) + self.b.sample(t)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        self.a.sample_with_context(t, ctx) + self.b.sample_with_context(t, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_add_constants() {
        let a = Constant::new(0.3);
        let b = Constant::new(0.4);
        let sum = Add::new(a, b);
        assert!((sum.sample(0.0) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_add_sine_offset() {
        let sine = Sine::with_frequency(1.0);
        let offset = Constant::new(0.2);
        let sum = Add::new(sine, offset);
        // At t=0, bipolar sine is 0, so sum should be 0.2
        assert!((sum.sample(0.0) - 0.2).abs() < 0.001);
        // At t=0.25, bipolar sine is 1.0, so sum is 1.2 (no clamping)
        assert!((sum.sample(0.25) - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_add_output_range() {
        let a = Constant::new(0.3);
        let b = Constant::new(0.4);
        let sum = Add::new(a, b);
        let range = sum.output_range();
        assert!((range.min - 0.7).abs() < 0.001);
        assert!((range.max - 0.7).abs() < 0.001);

        // Bipolar + constant
        let sine = Sine::default();
        let offset = Constant::new(0.5);
        let sum2 = Add::new(sine, offset);
        let range2 = sum2.output_range();
        assert!((range2.min - (-0.5)).abs() < 0.001);
        assert!((range2.max - 1.5).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/composition/cls_add.rs</FILE> - <DESC>Signal addition operator</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
