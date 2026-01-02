// <FILE>mixed-signals/src/processing/cls_quantize.rs</FILE> - <DESC>Signal quantization operator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor</WCTX>
// <CLOG>Removed input clamping, added output_range() - quantizes within signal's range</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Quantizes a signal to discrete levels within its range.
///
/// Useful for creating stepped/staircase effects or bit-crushing.
/// Works with any signal range - quantizes values within [min, max] to discrete steps.
#[derive(Debug, Clone)]
pub struct Quantize<S> {
    pub signal: S,
    /// Number of discrete levels
    pub levels: u8,
}

impl<S: Signal> Quantize<S> {
    pub fn new(signal: S, levels: u8) -> Self {
        Self {
            signal,
            levels: levels.max(2),
        }
    }
}

impl<S: Signal> Signal for Quantize<S> {
    fn output_range(&self) -> SignalRange {
        // Output range is same as input - we just snap to discrete values within it
        self.signal.output_range()
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let range = self.signal.output_range();
        let value = self.signal.sample(t);
        quantize_in_range(value, range, self.levels)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let range = self.signal.output_range();
        let value = self.signal.sample_with_context(t, ctx);
        quantize_in_range(value, range, self.levels)
    }
}

/// Quantize a value within a range to discrete levels
fn quantize_in_range(value: f32, range: SignalRange, levels: u8) -> f32 {
    let span = range.max - range.min;
    if span == 0.0 || levels < 2 {
        return value;
    }
    // Normalize to [0, 1], quantize, then map back
    let normalized = (value - range.min) / span;
    let step = 1.0 / (levels - 1) as f32;
    let quantized = (normalized / step).floor() * step;
    range.min + quantized * span
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_quantize_2_levels() {
        // Constant(0.3) has range [0.3, 0.3], quantizing to 2 levels
        // keeps the same value since span is 0
        let sig = Constant::new(0.3);
        let quantized = Quantize::new(sig, 2);
        let v = quantized.sample(0.0);
        assert!((v - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_quantize_bipolar_sine() {
        // Bipolar sine [-1, 1] with 3 levels: -1, 0, 1
        let sine = Sine::with_frequency(1.0);
        let quantized = Quantize::new(sine, 3);

        // At t=0, sin=0 -> quantizes to 0
        assert!((quantized.sample(0.0) - 0.0).abs() < 0.001);
        // At t=0.25, sin=1 -> quantizes to 1
        assert!((quantized.sample(0.25) - 1.0).abs() < 0.001);
        // At t=0.75, sin=-1 -> quantizes to -1
        assert!((quantized.sample(0.75) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_quantize_output_range() {
        // Output range should match input range
        let sine = Sine::with_frequency(1.0);
        let quantized = Quantize::new(sine, 5);
        let range = quantized.output_range();
        assert!((range.min - (-1.0)).abs() < 0.001);
        assert!((range.max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quantize_mid_value_unit_signal() {
        // Create a unit-range signal for testing old behavior
        struct UnitSignal(f32);
        impl Signal for UnitSignal {
            fn output_range(&self) -> SignalRange {
                SignalRange::UNIT
            }
            fn sample(&self, _t: SignalTime) -> f32 {
                self.0
            }
        }

        let mid = UnitSignal(0.5);
        let quantized = Quantize::new(mid, 3);
        // With 3 levels in [0, 1]: 0, 0.5, 1 -> 0.5 stays 0.5
        assert!((quantized.sample(0.0) - 0.5).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/processing/cls_quantize.rs</FILE> - <DESC>Signal quantization operator</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
