// <FILE>src/physics/cls_spring.rs</FILE> - <DESC>Damped spring harmonic motion solver</DESC>
// <VERS>VERSION: 1.2.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Use harmonic_sin_cos helper for trig stability</CLOG>

use crate::math::{finite_or, finite_or_f64, finite_or_min, harmonic_sin_cos};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Damped spring harmonic motion solver.
///
/// Models a mass-spring-damper system with analytical solution for position over time.
/// Handles three damping regimes:
/// - **Underdamped** (oscillates): spring bounces before settling
/// - **Critically damped**: fastest approach without overshoot
/// - **Overdamped** (sluggish): slow approach without oscillation
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the displacement from equilibrium
/// at time `t` (seconds). Output range is dynamic based on initial conditions.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::DampedSpring;
/// use mixed_signals::traits::Signal;
///
/// // Bouncy spring (underdamped)
/// let spring = DampedSpring::new(1.0, 100.0, 5.0, 0.0, 1.0);
/// let pos = spring.sample(0.5); // Position at t=0.5s
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DampedSpring {
    /// Mass of the object (kg). Must be positive; defaults to 1.0.
    pub mass: f32,
    /// Spring stiffness constant (k). Higher = stiffer spring.
    pub stiffness: f32,
    /// Damping coefficient (c). Higher = more friction.
    pub damping: f32,
    /// Initial velocity (units/sec).
    pub v0: f32,
    /// Initial displacement from equilibrium.
    pub x0: f32,
}

impl DampedSpring {
    /// Create a new damped spring with all parameters.
    pub fn new(mass: f32, stiffness: f32, damping: f32, v0: f32, x0: f32) -> Self {
        Self {
            mass,
            stiffness,
            damping,
            v0,
            x0,
        }
    }

    /// Create a spring with given stiffness, default mass (1.0), no damping.
    /// Initial displacement of 1.0 and no initial velocity.
    pub fn with_stiffness(stiffness: f32) -> Self {
        Self::new(1.0, stiffness, 0.0, 0.0, 1.0)
    }

    /// Create a critically damped spring for the smoothest animation.
    /// Critical damping: c = 2 * sqrt(k * m)
    pub fn critically_damped(stiffness: f32, x0: f32) -> Self {
        let mass = 1.0;
        let critical_damping = 2.0 * (stiffness * mass).sqrt();
        Self::new(mass, stiffness, critical_damping, 0.0, x0)
    }

    /// Analytical solution for position at time t.
    pub fn position_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);
        if t == 0.0 {
            return finite_or(self.x0, 0.0);
        }

        let m = finite_or_min(self.mass, 0.001, 1.0) as f64;
        let k = finite_or(self.stiffness, 100.0).max(0.0) as f64;
        let c = finite_or(self.damping, 0.0).max(0.0) as f64;
        let v0 = finite_or(self.v0, 0.0) as f64;
        let x0 = finite_or(self.x0, 0.0) as f64;

        // Discriminant for characteristic equation: m*r² + c*r + k = 0
        let discriminant = c * c - 4.0 * m * k;
        let gamma = c / (2.0 * m); // Damping ratio factor

        let result = if discriminant < -1e-10 {
            // Underdamped: oscillates
            let omega = (k / m - gamma * gamma).sqrt();
            let exp_term = (-gamma * t).exp();
            let a = x0;
            let b = (v0 + gamma * x0) / omega;
            let (sin_p, cos_p) = harmonic_sin_cos(omega, t, 0.0);
            exp_term * (a * cos_p + b * sin_p)
        } else if discriminant > 1e-10 {
            // Overdamped: sluggish
            let sqrt_disc = discriminant.sqrt();
            let r1 = (-c + sqrt_disc) / (2.0 * m);
            let r2 = (-c - sqrt_disc) / (2.0 * m);
            let denom = r2 - r1;
            if denom.abs() < 1e-10 {
                // Degenerate case, treat as critically damped
                (x0 + (v0 + gamma * x0) * t) * (-gamma * t).exp()
            } else {
                let a = (x0 * r2 - v0) / denom;
                let b = x0 - a;
                a * (r1 * t).exp() + b * (r2 * t).exp()
            }
        } else {
            // Critically damped: fastest non-oscillating
            (x0 + (v0 + gamma * x0) * t) * (-gamma * t).exp()
        };

        result as f32
    }

    /// Velocity at time t (derivative of position).
    pub fn velocity_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);
        if t == 0.0 {
            return finite_or(self.v0, 0.0);
        }

        let m = finite_or_min(self.mass, 0.001, 1.0) as f64;
        let k = finite_or(self.stiffness, 100.0).max(0.0) as f64;
        let c = finite_or(self.damping, 0.0).max(0.0) as f64;
        let v0 = finite_or(self.v0, 0.0) as f64;
        let x0 = finite_or(self.x0, 0.0) as f64;

        let discriminant = c * c - 4.0 * m * k;
        let gamma = c / (2.0 * m);

        let result = if discriminant < -1e-10 {
            // Underdamped
            let omega = (k / m - gamma * gamma).sqrt();
            let exp_term = (-gamma * t).exp();
            let a = x0;
            let b = (v0 + gamma * x0) / omega;
            // Derivative: d/dt[e^(-γt) * (A*cos(ωt) + B*sin(ωt))]
            let (sin_p, cos_p) = harmonic_sin_cos(omega, t, 0.0);
            exp_term * ((-gamma * a + omega * b) * cos_p + (-gamma * b - omega * a) * sin_p)
        } else if discriminant > 1e-10 {
            // Overdamped
            let sqrt_disc = discriminant.sqrt();
            let r1 = (-c + sqrt_disc) / (2.0 * m);
            let r2 = (-c - sqrt_disc) / (2.0 * m);
            let denom = r2 - r1;
            if denom.abs() < 1e-10 {
                let term = v0 + gamma * x0;
                (-gamma * t).exp() * (term - gamma * (x0 + term * t))
            } else {
                let a = (x0 * r2 - v0) / denom;
                let b = x0 - a;
                a * r1 * (r1 * t).exp() + b * r2 * (r2 * t).exp()
            }
        } else {
            // Critically damped
            let term = v0 + gamma * x0;
            (-gamma * t).exp() * (term - gamma * (x0 + term * t))
        };

        result as f32
    }
}

impl Default for DampedSpring {
    fn default() -> Self {
        // Slightly underdamped spring, natural for UI
        Self {
            mass: 1.0,
            stiffness: 100.0,
            damping: 10.0,
            v0: 0.0,
            x0: 1.0,
        }
    }
}

impl Signal for DampedSpring {
    fn output_range(&self) -> SignalRange {
        // For underdamped springs, can overshoot. Estimate max amplitude.
        let x0 = finite_or(self.x0, 0.0).abs();
        let v0 = finite_or(self.v0, 0.0).abs();
        let max_amplitude = x0 + v0 * 0.5; // Conservative estimate
        SignalRange::new(-max_amplitude, max_amplitude)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.position_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_spring_at_zero() {
        let spring = DampedSpring::new(1.0, 100.0, 10.0, 0.0, 1.0);
        assert!((spring.sample(0.0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_spring_converges_to_zero() {
        let spring = DampedSpring::default();
        // After sufficient time, should be near equilibrium
        let pos = spring.sample(5.0);
        assert!(pos.abs() < 0.01, "Expected near zero, got {}", pos);
    }

    #[test]
    fn test_critically_damped_no_overshoot() {
        let spring = DampedSpring::critically_damped(100.0, 1.0);
        // Critically damped should never go negative (overshoot past 0)
        for i in 1..100 {
            let t = i as f64 * 0.01;
            let pos = spring.sample(t);
            assert!(pos >= -EPSILON, "Overshoot at t={}: {}", t, pos);
        }
    }

    #[test]
    fn test_underdamped_oscillates() {
        // Low damping relative to stiffness
        let spring = DampedSpring::new(1.0, 100.0, 2.0, 0.0, 1.0);
        // Should cross zero (go negative) at some point
        let mut crossed_zero = false;
        for i in 1..200 {
            let t = i as f64 * 0.01;
            if spring.sample(t) < 0.0 {
                crossed_zero = true;
                break;
            }
        }
        assert!(
            crossed_zero,
            "Underdamped spring should oscillate past zero"
        );
    }

    #[test]
    fn test_overdamped_slow_approach() {
        // High damping
        let spring = DampedSpring::new(1.0, 100.0, 50.0, 0.0, 1.0);
        // At t=0.1, should still be relatively far from zero
        let pos = spring.sample(0.1);
        assert!(pos > 0.5, "Overdamped should approach slowly, got {}", pos);
    }

    #[test]
    fn test_initial_velocity() {
        let spring = DampedSpring::new(1.0, 100.0, 10.0, 10.0, 0.0);
        // With x0=0 and positive velocity, should move away from zero initially
        let pos = spring.sample(0.01);
        assert!(pos > 0.0, "Should move with initial velocity");
    }

    #[test]
    fn test_velocity_at_zero() {
        let spring = DampedSpring::new(1.0, 100.0, 10.0, 5.0, 1.0);
        assert!((spring.velocity_at(0.0) - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_output_range() {
        let spring = DampedSpring::default();
        let range = spring.output_range();
        assert!(range.min < 0.0);
        assert!(range.max > 0.0);
    }

    #[test]
    fn test_nan_handling() {
        let spring = DampedSpring::new(f32::NAN, 100.0, 10.0, 0.0, 1.0);
        let pos = spring.sample(0.5);
        assert!(pos.is_finite(), "Should handle NaN mass");
    }

    #[test]
    fn test_zero_stiffness() {
        let spring = DampedSpring::new(1.0, 0.0, 1.0, 0.0, 1.0);
        // With zero stiffness, just decays with damping
        let pos = spring.sample(1.0);
        assert!(pos.is_finite());
    }
}

// <FILE>src/physics/cls_spring.rs</FILE> - <DESC>Damped spring harmonic motion solver</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
