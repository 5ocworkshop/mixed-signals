// <FILE>mixed-signals/src/generators/cls_sawtooth.rs</FILE> - <DESC>Sawtooth wave oscillator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Changed to bipolar output [-1, 1], removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Sawtooth wave oscillator.
///
/// Produces a linear ramp from -1 to 1, then instant reset.
/// Output is bipolar [-1, 1] scaled by amplitude and shifted by offset.
///
/// Use `.normalized()` to convert to [0, 1] for TUI animations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sawtooth {
    /// Frequency in Hz (cycles per second)
    pub frequency: f32,
    /// Output amplitude (scales the 0..1 range)
    pub amplitude: f32,
    /// DC offset (shifts the output)
    pub offset: f32,
    /// Phase shift (normalized 0..1)
    pub phase: f32,
    /// If true, ramp goes from 1 to 0 (inverse sawtooth)
    pub inverted: bool,
}

impl Sawtooth {
    pub fn new(frequency: f32, amplitude: f32, offset: f32, phase: f32, inverted: bool) -> Self {
        Self {
            frequency,
            amplitude,
            offset,
            phase,
            inverted,
        }
    }

    pub fn with_frequency(frequency: f32) -> Self {
        Self::new(frequency, 1.0, 0.0, 0.0, false)
    }

    pub fn inverted(frequency: f32) -> Self {
        Self::new(frequency, 1.0, 0.0, 0.0, true)
    }
}

impl Default for Sawtooth {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            phase: 0.0,
            inverted: false,
        }
    }
}

impl Signal for Sawtooth {
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

        let cycle_pos = (t * frequency + phase).rem_euclid(1.0);
        // Bipolar ramp: -1 to +1
        let bipolar = if self.inverted {
            1.0 - 2.0 * cycle_pos
        } else {
            2.0 * cycle_pos - 1.0
        };
        (offset + amplitude * bipolar) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sawtooth_at_zero() {
        let saw = Sawtooth::default();
        // Bipolar: starts at -1
        assert!((saw.sample(0.0) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_sawtooth_at_half() {
        let saw = Sawtooth::with_frequency(1.0);
        // Bipolar: midpoint is 0
        assert!((saw.sample(0.5) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_sawtooth_near_end() {
        let saw = Sawtooth::with_frequency(1.0);
        // Just before reset, bipolar: 2*0.99 - 1 = 0.98
        assert!((saw.sample(0.99) - 0.98).abs() < 0.01);
    }

    #[test]
    fn test_sawtooth_inverted() {
        let saw = Sawtooth::inverted(1.0);
        // Inverted bipolar: starts at +1, goes to -1
        assert!((saw.sample(0.0) - 1.0).abs() < 0.001);
        assert!((saw.sample(0.5) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_sawtooth_periodic() {
        let saw = Sawtooth::default();
        assert!((saw.sample(0.0) - saw.sample(1.0)).abs() < 0.001);
    }

    #[test]
    fn test_sawtooth_output_range() {
        let saw = Sawtooth::default();
        let range = saw.output_range();
        assert_eq!(range.min, -1.0);
        assert_eq!(range.max, 1.0);
    }
}

// <FILE>mixed-signals/src/generators/cls_sawtooth.rs</FILE> - <DESC>Sawtooth wave oscillator</DESC>
// <VERS>END OF VERSION: 1.3.0</VERS>
