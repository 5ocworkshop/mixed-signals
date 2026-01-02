// <FILE>mixed-signals/src/generators/cls_triangle.rs</FILE> - <DESC>Triangle wave oscillator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Changed to bipolar output [-1, 1], removed clamping, added output_range()</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Triangle wave oscillator.
///
/// Produces a linear ramp up then linear ramp down pattern.
/// Output is bipolar [-1, 1] scaled by amplitude and shifted by offset.
///
/// Use `.normalized()` to convert to [0, 1] for TUI animations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// Frequency in Hz (cycles per second)
    pub frequency: f32,
    /// Output amplitude (scales the 0..1 range)
    pub amplitude: f32,
    /// DC offset (shifts the output)
    pub offset: f32,
    /// Phase shift (normalized 0..1)
    pub phase: f32,
}

impl Triangle {
    pub fn new(frequency: f32, amplitude: f32, offset: f32, phase: f32) -> Self {
        Self {
            frequency,
            amplitude,
            offset,
            phase,
        }
    }

    pub fn with_frequency(frequency: f32) -> Self {
        Self::new(frequency, 1.0, 0.0, 0.0)
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            frequency: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            phase: 0.0,
        }
    }
}

impl Signal for Triangle {
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

        // Normalized position in cycle (0..1)
        let cycle_pos = (t * frequency + phase).rem_euclid(1.0);

        // Triangle wave in bipolar [-1, 1]: ramp -1->1, ramp 1->-1
        let bipolar = if cycle_pos < 0.5 {
            4.0 * cycle_pos - 1.0
        } else {
            3.0 - 4.0 * cycle_pos
        };

        (offset + amplitude * bipolar) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_at_zero() {
        let tri = Triangle::default();
        // Bipolar: starts at -1
        assert!((tri.sample(0.0) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_triangle_at_quarter() {
        let tri = Triangle::with_frequency(1.0);
        // At t=0.25 (1/4 cycle), halfway up → bipolar 0
        assert!((tri.sample(0.25) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_triangle_at_half() {
        let tri = Triangle::with_frequency(1.0);
        // At t=0.5 (1/2 cycle), peak → bipolar +1
        assert!((tri.sample(0.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_triangle_at_three_quarters() {
        let tri = Triangle::with_frequency(1.0);
        // At t=0.75 (3/4 cycle), halfway down → bipolar 0
        assert!((tri.sample(0.75) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_triangle_periodic() {
        let tri = Triangle::default();
        assert!((tri.sample(0.0) - tri.sample(1.0)).abs() < 0.001);
        assert!((tri.sample(0.25) - tri.sample(1.25)).abs() < 0.001);
    }

    #[test]
    fn test_triangle_output_range() {
        let tri = Triangle::default();
        let range = tri.output_range();
        assert_eq!(range.min, -1.0);
        assert_eq!(range.max, 1.0);
    }
}

// <FILE>mixed-signals/src/generators/cls_triangle.rs</FILE> - <DESC>Triangle wave oscillator</DESC>
// <VERS>END OF VERSION: 1.3.0</VERS>
