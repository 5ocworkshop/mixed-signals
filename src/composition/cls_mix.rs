// <FILE>mixed-signals/src/composition/cls_mix.rs</FILE> - <DESC>Signal mixing/crossfade operator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Mixes two signals with a blend factor.
///
/// Output = a * (1 - mix) + b * mix (no clamping)
///
/// When mix = 0, output is entirely signal A.
/// When mix = 1, output is entirely signal B.
///
/// Use `.normalized()` to clamp result to [0, 1].
#[derive(Debug, Clone)]
pub struct Mix<A, B> {
    pub a: A,
    pub b: B,
    /// Blend factor (0 = all A, 1 = all B)
    pub mix: f32,
}

impl<A: Signal, B: Signal> Mix<A, B> {
    pub fn new(a: A, b: B, mix: f32) -> Self {
        let mix = if mix.is_finite() { mix } else { 0.5 };
        Self {
            a,
            b,
            mix: mix.clamp(0.0, 1.0),
        }
    }

    /// Equal mix of both signals (50/50)
    pub fn equal(a: A, b: B) -> Self {
        Self::new(a, b, 0.5)
    }
}

impl<A: Signal, B: Signal> Signal for Mix<A, B> {
    fn output_range(&self) -> SignalRange {
        let ra = self.a.output_range();
        let rb = self.b.output_range();
        let inv_mix = 1.0 - self.mix;
        // Lerp the range bounds
        let min = ra.min * inv_mix + rb.min * self.mix;
        let max = ra.max * inv_mix + rb.max * self.mix;
        SignalRange::new(min, max)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let va = self.a.sample(t);
        let vb = self.b.sample(t);
        va * (1.0 - self.mix) + vb * self.mix
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let va = self.a.sample_with_context(t, ctx);
        let vb = self.b.sample_with_context(t, ctx);
        va * (1.0 - self.mix) + vb * self.mix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    #[test]
    fn test_mix_all_a() {
        let a = Constant::new(0.2);
        let b = Constant::new(0.8);
        let mixed = Mix::new(a, b, 0.0);
        assert!((mixed.sample(0.0) - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_mix_all_b() {
        let a = Constant::new(0.2);
        let b = Constant::new(0.8);
        let mixed = Mix::new(a, b, 1.0);
        assert!((mixed.sample(0.0) - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_mix_equal() {
        let a = Constant::new(0.2);
        let b = Constant::new(0.8);
        let mixed = Mix::equal(a, b);
        assert!((mixed.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mix_quarter() {
        let a = Constant::new(0.0);
        let b = Constant::new(1.0);
        let mixed = Mix::new(a, b, 0.25);
        assert!((mixed.sample(0.0) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_mix_nan_defaults_to_equal() {
        let a = Constant::new(0.2);
        let b = Constant::new(0.8);
        let mixed = Mix::new(a, b, f32::NAN);
        assert!((mixed.sample(0.0) - 0.5).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/composition/cls_mix.rs</FILE> - <DESC>Signal mixing/crossfade operator</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
