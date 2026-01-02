// <FILE>mixed-signals/src/generators/cls_square.rs</FILE> - <DESC>Square wave oscillator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Changed to bipolar output [-1, 1], removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_clamp, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Square wave oscillator.
///
/// Produces a binary on/off pulse pattern.
/// Output is bipolar [-1, 1] scaled by amplitude and shifted by offset.
///
/// Use `.normalized()` to convert to [0, 1] for TUI animations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Square {
    /// Frequency in Hz (cycles per second)
    pub frequency: f32,
    /// Output amplitude (high = offset + amplitude, low = offset)
    pub amplitude: f32,
    /// DC offset (shifts the output)
    pub offset: f32,
    /// Phase shift (normalized 0..1)
    pub phase: f32,
    /// Duty cycle (0..1, default 0.5 for 50% high/low)
    pub duty: f32,
}

impl Square {
    pub fn new(frequency: f32, amplitude: f32, offset: f32, phase: f32, duty: f32) -> Self {
        Self {
            frequency,
            amplitude,
            offset,
            phase,
            duty: duty.clamp(0.0, 1.0),
        }
    }

    pub fn with_frequency(frequency: f32) -> Self {
        Self::new(frequency, 1.0, 0.0, 0.0, 0.5)
    }
}

impl Default for Square {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            phase: 0.0,
            duty: 0.5,
        }
    }
}

impl Signal for Square {
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
        let duty = finite_or_clamp(self.duty, 0.0, 1.0, 0.5) as f64;

        let cycle_pos = (t * frequency + phase).rem_euclid(1.0);
        // Bipolar: +1 during duty, -1 otherwise
        let bipolar = if cycle_pos < duty { 1.0 } else { -1.0 };
        (offset + amplitude * bipolar) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_high_phase() {
        let sq = Square::default();
        // At t=0, should be high (bipolar +1)
        assert!((sq.sample(0.0) - 1.0).abs() < 0.001);
        // At t=0.25, should still be high
        assert!((sq.sample(0.25) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_square_low_phase() {
        let sq = Square::default();
        // At t=0.5, should be low (bipolar -1)
        assert!((sq.sample(0.5) - (-1.0)).abs() < 0.001);
        // At t=0.75, should still be low
        assert!((sq.sample(0.75) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_square_duty_cycle() {
        let sq = Square::new(1.0, 1.0, 0.0, 0.0, 0.25);
        // With 25% duty, high (bipolar +1) for first quarter
        assert!((sq.sample(0.1) - 1.0).abs() < 0.001);
        // Low (bipolar -1) for remaining 75%
        assert!((sq.sample(0.3) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_square_periodic() {
        let sq = Square::default();
        assert!((sq.sample(0.0) - sq.sample(1.0)).abs() < 0.001);
    }

    #[test]
    fn test_square_output_range() {
        let sq = Square::default();
        let range = sq.output_range();
        assert_eq!(range.min, -1.0);
        assert_eq!(range.max, 1.0);
    }
}

// <FILE>mixed-signals/src/generators/cls_square.rs</FILE> - <DESC>Square wave oscillator</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
