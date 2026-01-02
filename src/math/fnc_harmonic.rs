// <FILE>src/math/fnc_harmonic.rs</FILE> - <DESC>Harmonic phase helpers for trig stability</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Initial implementation - extracted from physics solvers</CLOG>

use std::f64::consts::TAU;

/// Compute sin and cos of harmonic phase with stability for large t.
///
/// Wraps `omega * t + phase` to [0, TAU) before computing trig functions
/// to prevent precision loss when t is large.
///
/// # Arguments
///
/// * `omega` - Angular frequency (radians/sec)
/// * `t` - Time value
/// * `phase` - Initial phase offset (radians), use 0.0 if none
///
/// # Returns
///
/// `(sin, cos)` tuple of the wrapped phase angle
///
/// # Example
///
/// ```rust
/// use mixed_signals::math::harmonic_sin_cos;
///
/// // 1 Hz oscillation at t=0.25s (quarter cycle)
/// let omega = std::f64::consts::TAU; // 2π rad/s = 1 Hz
/// let (sin, cos) = harmonic_sin_cos(omega, 0.25, 0.0);
/// assert!((sin - 1.0).abs() < 0.001); // sin(π/2) = 1
/// assert!((cos - 0.0).abs() < 0.001); // cos(π/2) = 0
/// ```
#[inline]
pub fn harmonic_sin_cos(omega: f64, t: f64, phase: f64) -> (f64, f64) {
    let angle = (omega * t + phase).rem_euclid(TAU);
    angle.sin_cos()
}

/// Compute wrapped harmonic phase angle for large t stability.
///
/// Wraps `omega * t + phase` to [0, TAU) to prevent precision loss.
///
/// # Arguments
///
/// * `omega` - Angular frequency (radians/sec)
/// * `t` - Time value
/// * `phase` - Initial phase offset (radians), use 0.0 if none
///
/// # Returns
///
/// The phase angle wrapped to [0, TAU)
#[inline]
pub fn harmonic_phase(omega: f64, t: f64, phase: f64) -> f64 {
    (omega * t + phase).rem_euclid(TAU)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_harmonic_sin_cos_zero() {
        let (sin, cos) = harmonic_sin_cos(1.0, 0.0, 0.0);
        assert!((sin - 0.0).abs() < 0.001);
        assert!((cos - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_harmonic_sin_cos_quarter_cycle() {
        // At t where omega*t = π/2
        let (sin, cos) = harmonic_sin_cos(TAU, 0.25, 0.0);
        assert!((sin - 1.0).abs() < 0.001);
        assert!((cos - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_harmonic_sin_cos_with_phase() {
        // Phase offset of π/2 means we start at peak
        let (sin, cos) = harmonic_sin_cos(TAU, 0.0, PI / 2.0);
        assert!((sin - 1.0).abs() < 0.001);
        assert!((cos - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_harmonic_sin_cos_large_t_stability() {
        // Very large t should still give valid results
        let t_small = 0.25;
        let t_large = 1_000_000.25; // Same fractional part

        let (sin1, cos1) = harmonic_sin_cos(TAU, t_small, 0.0);
        let (sin2, cos2) = harmonic_sin_cos(TAU, t_large, 0.0);

        // Results should be very close (both are at quarter cycle)
        assert!((sin1 - sin2).abs() < 0.001);
        assert!((cos1 - cos2).abs() < 0.001);
    }

    #[test]
    fn test_harmonic_phase_wrapping() {
        let phase = harmonic_phase(1.0, TAU * 2.5, 0.0);
        // Should be equivalent to 0.5 * TAU = π
        assert!((phase - PI).abs() < 0.001);
    }
}

// <FILE>src/math/fnc_harmonic.rs</FILE> - <DESC>Harmonic phase helpers for trig stability</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
