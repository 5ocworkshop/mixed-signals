// <FILE>mixed-signals/src/generators/cls_sine.rs</FILE> - <DESC>Sine wave oscillator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Changed to bipolar output [-1, 1], removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// Sine wave oscillator.
///
/// Produces a smooth periodic oscillation following the sine function.
/// Output is bipolar [-1, 1] scaled by amplitude and shifted by offset.
///
/// Formula: `output = offset + amplitude * sin(2π * (frequency * t + phase))`
///
/// Use `.normalized()` to convert to [0, 1] for TUI animations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sine {
    /// Frequency in Hz (cycles per second)
    pub frequency: f32,
    /// Output amplitude (scales the 0..1 range)
    pub amplitude: f32,
    /// DC offset (shifts the output)
    pub offset: f32,
    /// Phase shift (normalized 0..1)
    pub phase: f32,
}

impl Sine {
    pub fn new(frequency: f32, amplitude: f32, offset: f32, phase: f32) -> Self {
        Self {
            frequency,
            amplitude,
            offset,
            phase,
        }
    }

    /// Create a sine wave with default amplitude (1.0) and no offset/phase.
    pub fn with_frequency(frequency: f32) -> Self {
        Self::new(frequency, 1.0, 0.0, 0.0)
    }
}

impl Default for Sine {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            phase: 0.0,
        }
    }
}

impl Signal for Sine {
    fn output_range(&self) -> SignalRange {
        let amplitude = finite_or(self.amplitude, 1.0);
        let offset = finite_or(self.offset, 0.0);
        SignalRange::new(offset - amplitude, offset + amplitude)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let frequency = finite_or(self.frequency, 1.0) as f64;
        let amplitude = finite_or(self.amplitude, 1.0) as f64;
        let offset = finite_or(self.offset, 0.0) as f64;
        let phase = finite_or(self.phase, 0.0) as f64;

        let angle = TAU * (frequency * t + phase);
        (offset + amplitude * angle.sin()) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_at_zero() {
        let sine = Sine::default();
        // sin(0) = 0 → bipolar output 0
        assert!((sine.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_sine_at_quarter() {
        let sine = Sine::with_frequency(1.0);
        // At t=0.25 (1/4 cycle), sin(π/2) = 1 → bipolar output 1.0
        assert!((sine.sample(0.25) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sine_at_half() {
        let sine = Sine::with_frequency(1.0);
        // At t=0.5 (1/2 cycle), sin(π) = 0 → bipolar output 0.0
        assert!((sine.sample(0.5) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_sine_at_three_quarters() {
        let sine = Sine::with_frequency(1.0);
        // At t=0.75 (3/4 cycle), sin(3π/2) = -1 → bipolar output -1.0
        assert!((sine.sample(0.75) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_sine_amplitude() {
        let sine = Sine::new(1.0, 0.5, 0.0, 0.0);
        // At t=0.25, amplitude 0.5 → output 0.5
        assert!((sine.sample(0.25) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_sine_offset() {
        let sine = Sine::new(1.0, 0.5, 0.25, 0.0);
        // At t=0, sin(0) = 0 → 0.25 + 0.5*0 = 0.25
        assert!((sine.sample(0.0) - 0.25).abs() < 0.001);
        // At t=0.25, sin(π/2) = 1 → 0.25 + 0.5*1 = 0.75
        assert!((sine.sample(0.25) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_sine_frequency() {
        let sine = Sine::with_frequency(2.0);
        // At 2 Hz, t=0.125 is 1/4 cycle → sin(π/2) = 1
        assert!((sine.sample(0.125) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sine_phase_normalized() {
        let sine = Sine::new(1.0, 1.0, 0.0, 0.25);
        // Phase 0.25 means starting at peak → sin(π/2) = 1
        assert!((sine.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sine_output_range() {
        let sine = Sine::default();
        let range = sine.output_range();
        assert_eq!(range.min, -1.0);
        assert_eq!(range.max, 1.0);

        let sine2 = Sine::new(1.0, 0.5, 0.3, 0.0);
        let range2 = sine2.output_range();
        assert!((range2.min - (-0.2)).abs() < 0.001);
        assert!((range2.max - 0.8).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/generators/cls_sine.rs</FILE> - <DESC>Sine wave oscillator</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
