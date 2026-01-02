// <FILE>src/traits/ext_signal.rs</FILE> - <DESC>Fluent combinator methods for Signal</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Bipolar core refactor</WCTX>
// <CLOG>Fixed .invert() to use negation (-v), updated docstrings to remove stale clamp references</CLOG>

use super::{Signal, SignalContext, SignalRange, SignalTime};
use crate::composition::{Add, Mix, Multiply};
use crate::processing::Normalized;

/// Extension trait providing fluent combinator methods for signals.
///
/// This trait is automatically implemented for all types that implement `Signal`.
/// It allows chaining signal operations in a readable, iterator-like style.
///
/// # Example
///
/// ```rust
/// use mixed_signals::generators::{Sine, Constant};
/// use mixed_signals::traits::{Signal, SignalExt};
///
/// let signal = Sine::with_frequency(1.0)
///     .add(Constant::new(0.1))
///     .scale(0.8)
///     .mix(Constant::new(0.5), 0.3);
///
/// let value = signal.sample(0.0);
/// ```
pub trait SignalExt: Signal + Sized {
    /// Add another signal to this one.
    ///
    /// Output = self + other (unclamped)
    fn add<S: Signal>(self, other: S) -> Add<Self, S> {
        Add::new(self, other)
    }

    /// Multiply this signal by another.
    ///
    /// Output = self * other (unclamped)
    fn multiply<S: Signal>(self, other: S) -> Multiply<Self, S> {
        Multiply::new(self, other)
    }

    /// Scale this signal by a constant factor.
    ///
    /// Output = self * factor (unclamped)
    fn scale(self, factor: f32) -> Multiply<Self, crate::generators::Constant> {
        Multiply::new(self, crate::generators::Constant::new(factor))
    }

    /// Mix this signal with another using a blend factor.
    ///
    /// When mix = 0, output is entirely self.
    /// When mix = 1, output is entirely other.
    fn mix<S: Signal>(self, other: S, blend: f32) -> Mix<Self, S> {
        Mix::new(self, other, blend)
    }

    /// Apply a mapping function to the signal output.
    ///
    /// The function receives the signal value and should return a new value.
    /// Output is unclamped; use `.normalized()` if you need [0, 1].
    fn map<F: Fn(f32) -> f32 + Send + Sync + Clone>(self, f: F) -> Map<Self, F> {
        Map { signal: self, f }
    }

    /// Negate the signal (-value).
    ///
    /// For bipolar signals [-1, 1], this flips polarity (180° phase shift).
    /// Equivalent to wrapping in `Invert::new(signal)`.
    fn invert(self) -> Map<Self, fn(f32) -> f32> {
        Map {
            signal: self,
            f: |v| -v,
        }
    }

    /// Normalize the signal output to [0, 1] range.
    ///
    /// Uses the signal's `output_range()` to determine the source range,
    /// then remaps all values to [0, 1]. This is the primary API for
    /// TUI consumers who need normalized values from bipolar signals.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::generators::Sine;
    /// use mixed_signals::traits::{Signal, SignalExt};
    ///
    /// // Sine outputs bipolar [-1, 1], normalize to [0, 1]
    /// let opacity = Sine::with_frequency(1.0).normalized();
    /// let value = opacity.sample(0.0);  // 0.5 (center of normalized range)
    /// ```
    fn normalized(self) -> Normalized<Self> {
        Normalized::new(self)
    }

    /// Normalize the signal from a specific range to [0, 1].
    ///
    /// Use this when you know the actual range differs from what
    /// `output_range()` reports, or when chaining signals with
    /// dynamic ranges.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mixed_signals::generators::Constant;
    /// use mixed_signals::traits::{Signal, SignalExt, SignalRange};
    ///
    /// // Force interpretation as bipolar even if signal reports UNIT
    /// let sig = Constant::new(0.0).normalized_from(SignalRange::BIPOLAR);
    /// let value = sig.sample(0.0);  // 0.5
    /// ```
    fn normalized_from(self, from: SignalRange) -> NormalizedFrom<Self> {
        NormalizedFrom::new(self, from)
    }
}

// Blanket implementation for all Signal types
impl<T: Signal + Sized> SignalExt for T {}

/// A signal with a mapping function applied.
#[derive(Debug, Clone)]
pub struct Map<S, F> {
    signal: S,
    f: F,
}

impl<S: Signal, F: Fn(f32) -> f32 + Send + Sync> Signal for Map<S, F> {
    fn sample(&self, t: SignalTime) -> f32 {
        (self.f)(self.signal.sample(t))
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        (self.f)(self.signal.sample_with_context(t, ctx))
    }
}

/// A signal normalized from an explicit range to [0, 1].
#[derive(Debug, Clone)]
pub struct NormalizedFrom<S> {
    signal: S,
    from: SignalRange,
}

impl<S> NormalizedFrom<S> {
    /// Create a new normalized wrapper with explicit source range.
    pub fn new(signal: S, from: SignalRange) -> Self {
        Self { signal, from }
    }
}

impl<S: Signal> Signal for NormalizedFrom<S> {
    fn output_range(&self) -> SignalRange {
        SignalRange::UNIT
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let v = self.signal.sample(t);
        crate::processing::remap_range(v, self.from, SignalRange::UNIT).clamp(0.0, 1.0)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let v = self.signal.sample_with_context(t, ctx);
        crate::processing::remap_range(v, self.from, SignalRange::UNIT).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_add_fluent() {
        let sig = Constant::new(0.3).add(Constant::new(0.4));
        assert!((sig.sample(0.0) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_multiply_fluent() {
        let sig = Constant::new(0.5).multiply(Constant::new(0.8));
        assert!((sig.sample(0.0) - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_scale_fluent() {
        let sig = Constant::new(0.5).scale(0.5);
        assert!((sig.sample(0.0) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_mix_fluent() {
        let sig = Constant::new(0.0).mix(Constant::new(1.0), 0.5);
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_map_fluent() {
        let sig = Constant::new(0.5).map(|v| v * 2.0);
        assert!((sig.sample(0.0) - 1.0).abs() < 0.001); // 0.5 * 2.0 = 1.0 (unclamped)
    }

    #[test]
    fn test_invert_fluent() {
        // Negation: 0.3 → -0.3
        let sig = Constant::new(0.3).invert();
        assert!((sig.sample(0.0) - (-0.3)).abs() < 0.001);
    }

    #[test]
    fn test_chained_fluent() {
        // Bipolar sine at t=0 is 0
        let sig = Sine::with_frequency(1.0)
            .scale(0.5) // 0 * 0.5 = 0 at t=0
            .add(Constant::new(0.1)) // 0 + 0.1 = 0.1
            .invert(); // -0.1 (negation)
        assert!((sig.sample(0.0) - (-0.1)).abs() < 0.001);
    }

    #[test]
    fn test_map_with_context() {
        let sig = Constant::new(0.4).map(|v| v + 0.1);
        let ctx = SignalContext::new(0, 0);
        assert!((sig.sample_with_context(0.0, &ctx) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_normalized_fluent() {
        // Bipolar Sine normalized to unit range
        // At t=0, bipolar sine = 0, normalized = 0.5
        let sig = Sine::with_frequency(1.0).normalized();
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
        // At t=0.25, bipolar sine = 1, normalized = 1.0
        assert!((sig.sample(0.25) - 1.0).abs() < 0.001);
        // At t=0.75, bipolar sine = -1, normalized = 0.0
        assert!((sig.sample(0.75) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_normalized_from_bipolar() {
        // Bipolar 0.0 -> normalized 0.5
        let sig = Constant::new(0.0).normalized_from(SignalRange::BIPOLAR);
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_normalized_from_custom_range() {
        // Create a raw signal that returns 15.0 without clamping
        struct RawValue(f32);
        impl Signal for RawValue {
            fn output_range(&self) -> SignalRange {
                SignalRange::new(10.0, 20.0)
            }
            fn sample(&self, _t: SignalTime) -> f32 {
                self.0
            }
        }

        // Custom range [10, 20], value 15 -> normalized 0.5
        let sig = RawValue(15.0).normalized_from(SignalRange::new(10.0, 20.0));
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
    }
}

// <FILE>src/traits/ext_signal.rs</FILE> - <DESC>Fluent combinator methods for Signal</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
