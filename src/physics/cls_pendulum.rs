// <FILE>src/physics/cls_pendulum.rs</FILE> - <DESC>Simple pendulum oscillation solver</DESC>
// <VERS>VERSION: 1.2.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Use harmonic_phase/harmonic_sin_cos helpers for trig stability</CLOG>

use crate::math::{finite_or, finite_or_f64, finite_or_min, harmonic_phase, harmonic_sin_cos};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Simple pendulum oscillation solver.
///
/// Models pendulum motion using the small-angle approximation with optional damping.
/// Useful for hanging elements, swinging signs, or "hinge" animations.
///
/// # Physics Model
///
/// Uses damped harmonic motion: `θ(t) = θ₀ * e^(-γt) * cos(ωt)`
///
/// Where:
/// - `ω = √(g/L)` is the natural frequency
/// - `γ` is the damping coefficient
///
/// Valid for small angles (θ < ~15°).
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the angle in radians.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::SimplePendulum;
/// use mixed_signals::traits::Signal;
///
/// let pendulum = SimplePendulum::new(1.0, 9.8, 0.3, 0.05);
/// let angle = pendulum.sample(0.5); // Angle at t=0.5s
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimplePendulum {
    /// Pendulum length (meters).
    pub length: f32,
    /// Gravitational acceleration (m/s²). Default: 9.8.
    pub gravity: f32,
    /// Initial angle (radians). Keep small for accuracy.
    pub theta0: f32,
    /// Damping coefficient. 0 = no damping.
    pub damping: f32,
}

impl SimplePendulum {
    /// Create a new pendulum with all parameters.
    pub fn new(length: f32, gravity: f32, theta0: f32, damping: f32) -> Self {
        Self {
            length,
            gravity,
            theta0,
            damping,
        }
    }

    /// Standard Earth pendulum with given length and initial angle.
    pub fn earth(length: f32, theta0: f32) -> Self {
        Self::new(length, 9.8, theta0, 0.0)
    }

    /// Damped Earth pendulum.
    pub fn damped(length: f32, theta0: f32, damping: f32) -> Self {
        Self::new(length, 9.8, theta0, damping)
    }

    /// Natural frequency ω = √(g/L).
    pub fn natural_frequency(&self) -> f32 {
        let length = finite_or_min(self.length, 0.001, 1.0);
        let gravity = finite_or(self.gravity, 9.8).max(0.0);
        (gravity / length).sqrt()
    }

    /// Period T = 2π/ω = 2π√(L/g).
    pub fn period(&self) -> f32 {
        let omega = self.natural_frequency();
        if omega.abs() < 1e-6 {
            return f32::INFINITY;
        }
        2.0 * std::f32::consts::PI / omega
    }

    /// Angle at time t in radians.
    pub fn angle_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);

        let theta0 = finite_or(self.theta0, 0.0) as f64;
        let damping = finite_or(self.damping, 0.0).max(0.0) as f64;
        let omega = self.natural_frequency() as f64;

        // θ(t) = θ₀ * e^(-γt) * cos(ωt)
        let decay = (-damping * t).exp();
        let oscillation = harmonic_phase(omega, t, 0.0).cos();

        (theta0 * decay * oscillation) as f32
    }

    /// Angular velocity at time t in radians/sec.
    pub fn angular_velocity_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);

        let theta0 = finite_or(self.theta0, 0.0) as f64;
        let damping = finite_or(self.damping, 0.0).max(0.0) as f64;
        let omega = self.natural_frequency() as f64;

        // d/dt[θ₀ * e^(-γt) * cos(ωt)]
        // = θ₀ * e^(-γt) * (-γ*cos(ωt) - ω*sin(ωt))
        let (sin_p, cos_p) = harmonic_sin_cos(omega, t, 0.0);
        let decay = (-damping * t).exp();
        let derivative = -damping * cos_p - omega * sin_p;

        (theta0 * decay * derivative) as f32
    }
}

impl Default for SimplePendulum {
    fn default() -> Self {
        Self {
            length: 1.0,
            gravity: 9.8,
            theta0: 0.2, // ~11.5 degrees
            damping: 0.1,
        }
    }
}

impl Signal for SimplePendulum {
    fn output_range(&self) -> SignalRange {
        let theta0 = finite_or(self.theta0, 0.0).abs();
        SignalRange::new(-theta0, theta0)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.angle_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_angle_at_zero() {
        let pendulum = SimplePendulum::new(1.0, 9.8, 0.3, 0.0);
        assert!((pendulum.angle_at(0.0) - 0.3).abs() < EPSILON);
    }

    #[test]
    fn test_undamped_oscillation() {
        let pendulum = SimplePendulum::earth(1.0, 0.2);

        // At t=0, angle = theta0
        assert!((pendulum.angle_at(0.0) - 0.2).abs() < EPSILON);

        // After half period, should be at -theta0
        let half_period = pendulum.period() / 2.0;
        let angle = pendulum.angle_at(half_period as f64);
        assert!((angle - (-0.2)).abs() < 0.01);
    }

    #[test]
    fn test_period_calculation() {
        let pendulum = SimplePendulum::earth(1.0, 0.1);
        // T = 2π√(1/9.8) ≈ 2.006s
        let period = pendulum.period();
        assert!((period - 2.006).abs() < 0.01);
    }

    #[test]
    fn test_damping_reduces_amplitude() {
        let undamped = SimplePendulum::earth(1.0, 0.2);
        let damped = SimplePendulum::damped(1.0, 0.2, 0.5);

        let t = 2.0;
        let amp_undamped = undamped.angle_at(t).abs();
        let amp_damped = damped.angle_at(t).abs();

        assert!(
            amp_damped < amp_undamped,
            "Damped should have smaller amplitude"
        );
    }

    #[test]
    fn test_heavy_damping_quick_decay() {
        let pendulum = SimplePendulum::damped(1.0, 0.5, 2.0);
        let angle = pendulum.angle_at(5.0);
        assert!(
            angle.abs() < 0.01,
            "Should be nearly stopped after heavy damping"
        );
    }

    #[test]
    fn test_natural_frequency() {
        let pendulum = SimplePendulum::earth(1.0, 0.1);
        let omega = pendulum.natural_frequency();
        // ω = √(9.8/1.0) ≈ 3.13
        assert!((omega - 3.13).abs() < 0.01);
    }

    #[test]
    fn test_angular_velocity_at_zero() {
        let pendulum = SimplePendulum::earth(1.0, 0.2);
        // At t=0, velocity should be 0 (starting from rest)
        let vel = pendulum.angular_velocity_at(0.0);
        assert!(vel.abs() < 0.01);
    }

    #[test]
    fn test_output_range() {
        let pendulum = SimplePendulum::new(1.0, 9.8, 0.25, 0.0);
        let range = pendulum.output_range();
        assert!((range.min - (-0.25)).abs() < EPSILON);
        assert!((range.max - 0.25).abs() < EPSILON);
    }

    #[test]
    fn test_nan_handling() {
        let pendulum = SimplePendulum::new(f32::NAN, 9.8, 0.2, 0.0);
        let angle = pendulum.angle_at(1.0);
        assert!(angle.is_finite());
    }

    #[test]
    fn test_zero_length_fallback() {
        let pendulum = SimplePendulum::new(0.0, 9.8, 0.2, 0.0);
        let angle = pendulum.angle_at(1.0);
        assert!(angle.is_finite());
    }
}

// <FILE>src/physics/cls_pendulum.rs</FILE> - <DESC>Simple pendulum oscillation solver</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
