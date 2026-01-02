// <FILE>src/physics/cls_orbit.rs</FILE> - <DESC>Circular orbital motion solver</DESC>
// <VERS>VERSION: 1.2.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Use harmonic_phase/harmonic_sin_cos helpers for trig stability</CLOG>

use crate::math::{finite_or, finite_or_f64, harmonic_phase, harmonic_sin_cos};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// Circular orbital motion solver.
///
/// Models uniform circular motion around a center point.
/// Useful for loading spinners, satellite menu items, or rotating elements.
///
/// # Physics Model
///
/// Position: `(x, y) = (cx + r*cos(ωt + φ), cy + r*sin(ωt + φ))`
///
/// Where:
/// - `(cx, cy)` is the center
/// - `r` is the radius
/// - `ω` is angular velocity (radians/sec)
/// - `φ` is the starting phase
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the angle in radians.
/// Use `position_at()` for x,y coordinates.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::CircularOrbit;
/// use mixed_signals::traits::Signal;
///
/// let orbit = CircularOrbit::new(100.0, 100.0, 50.0, 2.0, 0.0);
/// let (x, y) = orbit.position_at(0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CircularOrbit {
    /// Center X coordinate.
    pub center_x: f32,
    /// Center Y coordinate.
    pub center_y: f32,
    /// Orbit radius.
    pub radius: f32,
    /// Angular velocity (radians/sec). Positive = counter-clockwise.
    pub angular_velocity: f32,
    /// Starting phase angle (radians).
    pub start_phase: f32,
}

impl CircularOrbit {
    /// Create a new circular orbit with all parameters.
    pub fn new(
        center_x: f32,
        center_y: f32,
        radius: f32,
        angular_velocity: f32,
        start_phase: f32,
    ) -> Self {
        Self {
            center_x,
            center_y,
            radius,
            angular_velocity,
            start_phase,
        }
    }

    /// Orbit centered at origin with given radius and angular velocity.
    pub fn centered(radius: f32, angular_velocity: f32) -> Self {
        Self::new(0.0, 0.0, radius, angular_velocity, 0.0)
    }

    /// Create an orbit that completes one revolution per second.
    pub fn one_hz(center_x: f32, center_y: f32, radius: f32) -> Self {
        Self::new(center_x, center_y, radius, TAU as f32, 0.0)
    }

    /// Current angle at time t (radians).
    /// Returns the wrapped angle in [0, TAU) for stable trig operations.
    pub fn angle_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);
        let omega = finite_or(self.angular_velocity, 1.0) as f64;
        let phase = finite_or(self.start_phase, 0.0) as f64;

        harmonic_phase(omega, t, phase) as f32
    }

    /// Unwrapped angle at time t (can exceed TAU for large t).
    /// Use this if you need to track total rotation.
    pub fn angle_unwrapped(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);
        let omega = finite_or(self.angular_velocity, 1.0) as f64;
        let phase = finite_or(self.start_phase, 0.0) as f64;
        (omega * t + phase) as f32
    }

    /// Position (x, y) at time t.
    pub fn position_at(&self, t: SignalTime) -> (f32, f32) {
        let t = finite_or_f64(t, 0.0).max(0.0);
        let omega = finite_or(self.angular_velocity, 1.0) as f64;
        let phase = finite_or(self.start_phase, 0.0) as f64;
        let cx = finite_or(self.center_x, 0.0) as f64;
        let cy = finite_or(self.center_y, 0.0) as f64;
        let r = finite_or(self.radius, 1.0).max(0.0) as f64;

        let (sin_a, cos_a) = harmonic_sin_cos(omega, t, phase);

        let x = cx + r * cos_a;
        let y = cy + r * sin_a;

        (x as f32, y as f32)
    }

    /// Velocity (vx, vy) at time t.
    pub fn velocity_at(&self, t: SignalTime) -> (f32, f32) {
        let t = finite_or_f64(t, 0.0).max(0.0);
        let omega = finite_or(self.angular_velocity, 1.0) as f64;
        let phase = finite_or(self.start_phase, 0.0) as f64;
        let r = finite_or(self.radius, 1.0).max(0.0) as f64;

        let (sin_a, cos_a) = harmonic_sin_cos(omega, t, phase);

        // Derivative of position: v = ω × r × (-sin, cos)
        let vx = -r * omega * sin_a;
        let vy = r * omega * cos_a;

        (vx as f32, vy as f32)
    }

    /// Period of one complete revolution.
    pub fn period(&self) -> f32 {
        let omega = finite_or(self.angular_velocity, 1.0).abs();
        if omega < 1e-6 {
            return f32::INFINITY;
        }
        TAU as f32 / omega
    }
}

impl Default for CircularOrbit {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            radius: 50.0,
            angular_velocity: TAU as f32, // 1 revolution per second
            start_phase: 0.0,
        }
    }
}

impl Signal for CircularOrbit {
    fn output_range(&self) -> SignalRange {
        // Wrapped angle is always in [0, TAU)
        SignalRange::new(0.0, TAU as f32)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.angle_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 0.01;

    #[test]
    fn test_angle_at_zero() {
        let orbit = CircularOrbit::new(0.0, 0.0, 50.0, 1.0, 0.5);
        assert!((orbit.angle_at(0.0) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_position_at_zero() {
        let orbit = CircularOrbit::new(100.0, 100.0, 50.0, 1.0, 0.0);
        let (x, y) = orbit.position_at(0.0);
        // At angle 0: x = 100 + 50*cos(0) = 150, y = 100 + 50*sin(0) = 100
        assert!((x - 150.0).abs() < EPSILON);
        assert!((y - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_position_at_quarter() {
        let orbit = CircularOrbit::centered(50.0, 1.0);
        // At angle π/2: x = 50*cos(π/2) = 0, y = 50*sin(π/2) = 50
        let (x, y) = orbit.position_at((PI / 2.0) as f64);
        assert!(x.abs() < EPSILON);
        assert!((y - 50.0).abs() < EPSILON);
    }

    #[test]
    fn test_one_hz_period() {
        let orbit = CircularOrbit::one_hz(0.0, 0.0, 50.0);
        assert!((orbit.period() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_negative_angular_velocity() {
        let orbit = CircularOrbit::new(0.0, 0.0, 50.0, -1.0, 0.0);
        // angle_at returns wrapped angle, use angle_unwrapped for raw
        let angle_unwrapped = orbit.angle_unwrapped(1.0);
        assert!(
            angle_unwrapped < 0.0,
            "Negative ω should give negative unwrapped angle"
        );

        // Wrapped angle should be in [0, TAU)
        let angle_wrapped = orbit.angle_at(1.0);
        assert!(angle_wrapped >= 0.0 && angle_wrapped < TAU as f32);
    }

    #[test]
    fn test_velocity_magnitude() {
        let orbit = CircularOrbit::centered(50.0, 2.0);
        let (vx, vy) = orbit.velocity_at(0.0);
        // At angle 0: vx = -r*ω*sin(0) = 0, vy = r*ω*cos(0) = 100
        assert!(vx.abs() < EPSILON);
        assert!((vy - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_full_revolution() {
        let orbit = CircularOrbit::centered(50.0, TAU as f32);
        let (x0, y0) = orbit.position_at(0.0);
        let (x1, y1) = orbit.position_at(1.0);
        // After one period, should be back at start
        assert!((x0 - x1).abs() < EPSILON);
        assert!((y0 - y1).abs() < EPSILON);
    }

    #[test]
    fn test_start_phase() {
        let orbit = CircularOrbit::new(0.0, 0.0, 50.0, 1.0, PI);
        let (x, _) = orbit.position_at(0.0);
        // At angle π: x = 50*cos(π) = -50
        assert!((x - (-50.0)).abs() < EPSILON);
    }

    #[test]
    fn test_nan_handling() {
        let orbit = CircularOrbit::new(f32::NAN, 100.0, 50.0, 1.0, 0.0);
        let (x, _) = orbit.position_at(1.0);
        assert!(x.is_finite());
    }

    #[test]
    fn test_zero_angular_velocity() {
        let orbit = CircularOrbit::new(0.0, 0.0, 50.0, 0.0, 0.0);
        let period = orbit.period();
        assert!(period.is_infinite());
    }
}

// <FILE>src/physics/cls_orbit.rs</FILE> - <DESC>Circular orbital motion solver</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
