// <FILE>mixed-signals/src/composition/cls_multiply.rs</FILE> - <DESC>Signal multiplication operator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Multiplies two signals together (ring modulation).
///
/// Output = signal_a * signal_b (no clamping)
///
/// Use `.normalized()` to clamp result to [0, 1].
#[derive(Debug, Clone)]
pub struct Multiply<A, B> {
    pub a: A,
    pub b: B,
}

impl<A: Signal, B: Signal> Multiply<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Signal, B: Signal> Signal for Multiply<A, B> {
    fn output_range(&self) -> SignalRange {
        let ra = self.a.output_range();
        let rb = self.b.output_range();
        // All 4 corner products to find min/max
        let products = [
            ra.min * rb.min,
            ra.min * rb.max,
            ra.max * rb.min,
            ra.max * rb.max,
        ];
        let min = products.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = products.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        SignalRange::new(min, max)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.a.sample(t) * self.b.sample(t)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        self.a.sample_with_context(t, ctx) * self.b.sample_with_context(t, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_multiply_constants() {
        let a = Constant::new(0.2);
        let b = Constant::new(0.3);
        let product = Multiply::new(a, b);
        assert!((product.sample(0.0) - 0.06).abs() < 0.001);
    }

    #[test]
    fn test_multiply_sine_by_constant() {
        let sine = Sine::with_frequency(1.0);
        let scale = Constant::new(0.5);
        let product = Multiply::new(sine, scale);
        // At t=0.25, bipolar sine is 1, scaled by 0.5
        assert!((product.sample(0.25) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_multiply_by_zero() {
        let sine = Sine::with_frequency(1.0);
        let zero = Constant::zero();
        let product = Multiply::new(sine, zero);
        assert!((product.sample(0.25) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_multiply_output_range() {
        // Bipolar [-1,1] * constant [0.5,0.5] = [-0.5, 0.5]
        let sine = Sine::default();
        let scale = Constant::new(0.5);
        let product = Multiply::new(sine, scale);
        let range = product.output_range();
        assert!((range.min - (-0.5)).abs() < 0.001);
        assert!((range.max - 0.5).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/composition/cls_multiply.rs</FILE> - <DESC>Signal multiplication operator</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
