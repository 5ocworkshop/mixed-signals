// <FILE>mixed-signals/src/processing/fnc_bipolar_helpers.rs</FILE> - <DESC>Bipolar signal conversion utilities</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Audio synthesis primitives - bipolar signal support</WCTX>
// <CLOG>Initial implementation - unipolar/bipolar conversion functions and SignalRange extensions</CLOG>

//! Utility functions for working with bipolar signals (-1..1 range).
//!
//! The library's default convention is unipolar (0..1), but audio synthesis
//! often requires bipolar signals. These helpers simplify conversions.

use crate::traits::SignalRange;

/// Convert a unipolar value (0..1) to bipolar (-1..1).
///
/// Formula: `value * 2.0 - 1.0`
///
/// # Examples
///
/// ```
/// use mixed_signals::processing::unipolar_to_bipolar;
///
/// assert_eq!(unipolar_to_bipolar(0.0), -1.0);
/// assert_eq!(unipolar_to_bipolar(0.5), 0.0);
/// assert_eq!(unipolar_to_bipolar(1.0), 1.0);
/// ```
#[inline]
pub fn unipolar_to_bipolar(value: f32) -> f32 {
    value * 2.0 - 1.0
}

/// Convert a bipolar value (-1..1) to unipolar (0..1).
///
/// Formula: `(value + 1.0) * 0.5`
///
/// # Examples
///
/// ```
/// use mixed_signals::processing::bipolar_to_unipolar;
///
/// assert_eq!(bipolar_to_unipolar(-1.0), 0.0);
/// assert_eq!(bipolar_to_unipolar(0.0), 0.5);
/// assert_eq!(bipolar_to_unipolar(1.0), 1.0);
/// ```
#[inline]
pub fn bipolar_to_unipolar(value: f32) -> f32 {
    (value + 1.0) * 0.5
}

/// Remap a value from one range to another.
///
/// Performs linear interpolation from the source range to the target range.
/// Handles zero-width source ranges by returning the target range's center.
///
/// # Examples
///
/// ```
/// use mixed_signals::processing::remap_range;
/// use mixed_signals::traits::SignalRange;
///
/// let from = SignalRange::UNIT;
/// let to = SignalRange::BIPOLAR;
/// assert_eq!(remap_range(0.5, from, to), 0.0);
/// ```
#[inline]
pub fn remap_range(value: f32, from: SignalRange, to: SignalRange) -> f32 {
    let from_width = from.max - from.min;
    if !from_width.is_finite() || from_width.abs() < f32::EPSILON {
        // Zero-width source range: return target center
        return (to.min + to.max) * 0.5;
    }
    let normalized = (value - from.min) / from_width;
    to.min + normalized * (to.max - to.min)
}

/// Extension methods for SignalRange.
impl SignalRange {
    /// Remap a value from this range to another range.
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_signals::traits::SignalRange;
    ///
    /// let unit = SignalRange::UNIT;
    /// let bipolar = SignalRange::BIPOLAR;
    /// assert_eq!(unit.remap_to(0.5, bipolar), 0.0);
    /// ```
    #[inline]
    pub fn remap_to(&self, value: f32, to: SignalRange) -> f32 {
        remap_range(value, *self, to)
    }

    /// Check if a value is within this range (inclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_signals::traits::SignalRange;
    ///
    /// let range = SignalRange::UNIT;
    /// assert!(range.contains(0.5));
    /// assert!(!range.contains(1.5));
    /// ```
    #[inline]
    pub fn contains(&self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }

    /// Clamp a value to this range.
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_signals::traits::SignalRange;
    ///
    /// let range = SignalRange::UNIT;
    /// assert_eq!(range.clamp_value(1.5), 1.0);
    /// assert_eq!(range.clamp_value(-0.5), 0.0);
    /// ```
    #[inline]
    pub fn clamp_value(&self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }

    /// Get the center (midpoint) of this range.
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_signals::traits::SignalRange;
    ///
    /// assert_eq!(SignalRange::UNIT.center(), 0.5);
    /// assert_eq!(SignalRange::BIPOLAR.center(), 0.0);
    /// ```
    #[inline]
    pub fn center(&self) -> f32 {
        (self.min + self.max) * 0.5
    }

    /// Get the width (span) of this range.
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_signals::traits::SignalRange;
    ///
    /// assert_eq!(SignalRange::UNIT.width(), 1.0);
    /// assert_eq!(SignalRange::BIPOLAR.width(), 2.0);
    /// ```
    #[inline]
    pub fn width(&self) -> f32 {
        self.max - self.min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unipolar_to_bipolar() {
        assert_eq!(unipolar_to_bipolar(0.0), -1.0);
        assert_eq!(unipolar_to_bipolar(0.5), 0.0);
        assert_eq!(unipolar_to_bipolar(1.0), 1.0);
        assert_eq!(unipolar_to_bipolar(0.25), -0.5);
        assert_eq!(unipolar_to_bipolar(0.75), 0.5);
    }

    #[test]
    fn test_bipolar_to_unipolar() {
        assert_eq!(bipolar_to_unipolar(-1.0), 0.0);
        assert_eq!(bipolar_to_unipolar(0.0), 0.5);
        assert_eq!(bipolar_to_unipolar(1.0), 1.0);
        assert_eq!(bipolar_to_unipolar(-0.5), 0.25);
        assert_eq!(bipolar_to_unipolar(0.5), 0.75);
    }

    #[test]
    fn test_roundtrip() {
        for i in 0..=10 {
            let v = i as f32 / 10.0;
            let roundtrip = bipolar_to_unipolar(unipolar_to_bipolar(v));
            assert!((roundtrip - v).abs() < 1e-6, "Roundtrip failed for {}", v);
        }
    }

    #[test]
    fn test_remap_range() {
        let unit = SignalRange::UNIT;
        let bipolar = SignalRange::BIPOLAR;

        assert_eq!(remap_range(0.0, unit, bipolar), -1.0);
        assert_eq!(remap_range(0.5, unit, bipolar), 0.0);
        assert_eq!(remap_range(1.0, unit, bipolar), 1.0);

        assert_eq!(remap_range(-1.0, bipolar, unit), 0.0);
        assert_eq!(remap_range(0.0, bipolar, unit), 0.5);
        assert_eq!(remap_range(1.0, bipolar, unit), 1.0);
    }

    #[test]
    fn test_remap_zero_width_range() {
        let zero_width = SignalRange::new(0.5, 0.5);
        let target = SignalRange::BIPOLAR;
        // Should return target center
        assert_eq!(remap_range(0.5, zero_width, target), 0.0);
    }

    #[test]
    fn test_signal_range_contains() {
        let range = SignalRange::UNIT;
        assert!(range.contains(0.0));
        assert!(range.contains(0.5));
        assert!(range.contains(1.0));
        assert!(!range.contains(-0.1));
        assert!(!range.contains(1.1));
    }

    #[test]
    fn test_signal_range_clamp() {
        let range = SignalRange::UNIT;
        assert_eq!(range.clamp_value(0.5), 0.5);
        assert_eq!(range.clamp_value(-0.5), 0.0);
        assert_eq!(range.clamp_value(1.5), 1.0);
    }

    #[test]
    fn test_signal_range_center() {
        assert_eq!(SignalRange::UNIT.center(), 0.5);
        assert_eq!(SignalRange::BIPOLAR.center(), 0.0);
        assert_eq!(SignalRange::new(0.0, 10.0).center(), 5.0);
    }

    #[test]
    fn test_signal_range_width() {
        assert_eq!(SignalRange::UNIT.width(), 1.0);
        assert_eq!(SignalRange::BIPOLAR.width(), 2.0);
        assert_eq!(SignalRange::new(0.0, 10.0).width(), 10.0);
    }

    #[test]
    fn test_nan_handling() {
        // NaN should propagate
        assert!(unipolar_to_bipolar(f32::NAN).is_nan());
        assert!(bipolar_to_unipolar(f32::NAN).is_nan());
    }
}

// <FILE>mixed-signals/src/processing/fnc_bipolar_helpers.rs</FILE> - <DESC>Bipolar signal conversion utilities</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
