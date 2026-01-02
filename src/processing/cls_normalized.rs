// <FILE>src/processing/cls_normalized.rs</FILE> - <DESC>Normalize any signal to 0..1 range</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Initial implementation - auto-normalizes using signal's output_range()</CLOG>

//! Signal normalization wrapper.
//!
//! The `Normalized` wrapper remaps any signal's output to the unit range [0, 1].
//! It uses the signal's `output_range()` to determine the source range, then
//! linearly remaps values to [0, 1] with clamping for safety.
//!
//! This is the primary API for TUI consumers who need normalized values.

use crate::processing::remap_range;
use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Wraps a signal and normalizes its output to [0, 1].
///
/// Uses the wrapped signal's `output_range()` to determine the source range,
/// then remaps all sampled values to [0, 1]. Output is clamped after remapping
/// to ensure values stay within bounds even if the source signal exceeds its
/// declared range.
///
/// # Examples
///
/// ```
/// use mixed_signals::generators::Sine;
/// use mixed_signals::processing::Normalized;
/// use mixed_signals::traits::Signal;
///
/// // Sine outputs bipolar [-1, 1], normalize to [0, 1]
/// let sine = Sine::default();
/// let normalized = Normalized::new(sine);
///
/// // At t=0, sin(0) = 0 in bipolar → 0.5 in normalized
/// let value = normalized.sample(0.0);
/// assert!((value - 0.5).abs() < 0.01);
/// ```
#[derive(Debug, Clone)]
pub struct Normalized<S> {
    signal: S,
}

impl<S> Normalized<S> {
    /// Create a new normalized wrapper around a signal.
    ///
    /// The wrapper will use the signal's `output_range()` to determine
    /// how to remap values to [0, 1].
    pub fn new(signal: S) -> Self {
        Self { signal }
    }

    /// Get a reference to the wrapped signal.
    pub fn inner(&self) -> &S {
        &self.signal
    }

    /// Unwrap and return the inner signal.
    pub fn into_inner(self) -> S {
        self.signal
    }
}

impl<S: Signal> Signal for Normalized<S> {
    fn output_range(&self) -> SignalRange {
        SignalRange::UNIT
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let v = self.signal.sample(t);
        let from_range = self.signal.output_range();
        remap_range(v, from_range, SignalRange::UNIT).clamp(0.0, 1.0)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let v = self.signal.sample_with_context(t, ctx);
        let from_range = self.signal.output_range();
        remap_range(v, from_range, SignalRange::UNIT).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper struct for testing bipolar signals
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
    fn test_normalized_bipolar_center() {
        // Bipolar 0.0 → normalized 0.5
        let sig = Normalized::new(BipolarConstant(0.0));
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_normalized_bipolar_min() {
        // Bipolar -1.0 → normalized 0.0
        let sig = Normalized::new(BipolarConstant(-1.0));
        assert!((sig.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_normalized_bipolar_max() {
        // Bipolar 1.0 → normalized 1.0
        let sig = Normalized::new(BipolarConstant(1.0));
        assert!((sig.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalized_unit_passthrough() {
        // Constant now has exact value range, not unit range
        // Constant(0.7) has output_range [0.7, 0.7], normalizing to unit
        // gives (0.7 - 0.7) / 0 = NaN, which clamps to 0.0
        // For a true passthrough test, use a unit-range signal
        struct UnitConstant(f32);
        impl Signal for UnitConstant {
            fn output_range(&self) -> SignalRange {
                SignalRange::UNIT
            }
            fn sample(&self, _t: SignalTime) -> f32 {
                self.0
            }
        }
        let sig = Normalized::new(UnitConstant(0.7));
        assert!((sig.sample(0.0) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_normalized_output_range() {
        let sig = Normalized::new(BipolarConstant(0.0));
        let range = sig.output_range();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.max, 1.0);
    }

    #[test]
    fn test_normalized_clamps_overflow() {
        // Value exceeds declared range → should be clamped
        struct OverflowSignal;
        impl Signal for OverflowSignal {
            fn output_range(&self) -> SignalRange {
                SignalRange::BIPOLAR
            }
            fn sample(&self, _t: SignalTime) -> f32 {
                2.0 // Exceeds [-1, 1]
            }
        }

        let sig = Normalized::new(OverflowSignal);
        assert_eq!(sig.sample(0.0), 1.0); // Clamped to max
    }

    #[test]
    fn test_normalized_clamps_underflow() {
        struct UnderflowSignal;
        impl Signal for UnderflowSignal {
            fn output_range(&self) -> SignalRange {
                SignalRange::BIPOLAR
            }
            fn sample(&self, _t: SignalTime) -> f32 {
                -2.0 // Exceeds [-1, 1]
            }
        }

        let sig = Normalized::new(UnderflowSignal);
        assert_eq!(sig.sample(0.0), 0.0); // Clamped to min
    }

    #[test]
    fn test_normalized_with_context() {
        let sig = Normalized::new(BipolarConstant(0.5));
        let ctx = SignalContext::default();
        // Bipolar 0.5 → normalized 0.75
        assert!((sig.sample_with_context(0.0, &ctx) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_inner_access() {
        let inner = BipolarConstant(0.5);
        let normalized = Normalized::new(inner);
        assert_eq!(normalized.inner().0, 0.5);
    }
}

// <FILE>src/processing/cls_normalized.rs</FILE> - <DESC>Normalize any signal to 0..1 range</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
