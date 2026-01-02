// <FILE>src/physics/cls_projectile.rs</FILE> - <DESC>Ballistic trajectory solver</DESC>
// <VERS>VERSION: 1.1.0</VERS>
// <WCTX>Physics module robustness improvements</WCTX>
// <CLOG>Remove Vec allocation in time_to_ground, consistent negative time handling</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Ballistic trajectory solver for parabolic motion under gravity.
///
/// Models projectile motion with optional ground collision detection.
/// Useful for "tossing" interactions or gravity-driven entrances.
///
/// # Coordinate System
///
/// Uses UI coordinates where positive Y is typically "down".
/// Gravity should be positive for objects falling down.
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the Y position at time t.
/// For X position or full coordinates, use `position_at()`.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::BallisticTrajectory;
///
/// // Toss an element upward
/// let toss = BallisticTrajectory::new(0.0, 100.0, 50.0, -200.0, 500.0, Some(500.0));
/// let (x, y) = toss.position_at(0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BallisticTrajectory {
    /// Starting X position.
    pub start_x: f32,
    /// Starting Y position.
    pub start_y: f32,
    /// Initial X velocity (units/sec).
    pub v0_x: f32,
    /// Initial Y velocity (units/sec). Negative = upward in UI coords.
    pub v0_y: f32,
    /// Gravity acceleration. Positive = down in UI coords.
    pub gravity: f32,
    /// Optional ground plane Y coordinate for collision.
    pub ground_y: Option<f32>,
}

impl BallisticTrajectory {
    /// Create a new ballistic trajectory with all parameters.
    pub fn new(
        start_x: f32,
        start_y: f32,
        v0_x: f32,
        v0_y: f32,
        gravity: f32,
        ground_y: Option<f32>,
    ) -> Self {
        Self {
            start_x,
            start_y,
            v0_x,
            v0_y,
            gravity,
            ground_y,
        }
    }

    /// Simple drop from a height with no horizontal velocity.
    pub fn drop_from(start_y: f32, ground_y: f32, gravity: f32) -> Self {
        Self::new(0.0, start_y, 0.0, 0.0, gravity, Some(ground_y))
    }

    /// Horizontal toss with given initial velocity.
    pub fn toss(start_x: f32, start_y: f32, v0_x: f32, v0_y: f32, gravity: f32) -> Self {
        Self::new(start_x, start_y, v0_x, v0_y, gravity, None)
    }

    /// Position (x, y) at time t.
    pub fn position_at(&self, t: SignalTime) -> (f32, f32) {
        let t = finite_or_f64(t, 0.0).max(0.0);

        let start_x = finite_or(self.start_x, 0.0) as f64;
        let start_y = finite_or(self.start_y, 0.0) as f64;
        let v0_x = finite_or(self.v0_x, 0.0) as f64;
        let v0_y = finite_or(self.v0_y, 0.0) as f64;
        let g = finite_or(self.gravity, 0.0) as f64;

        // Standard kinematic equations
        let x = start_x + v0_x * t;
        let y = start_y + v0_y * t + 0.5 * g * t * t;

        // Clamp to ground if specified
        let y = if let Some(ground) = self.ground_y {
            let ground = finite_or(ground, 0.0) as f64;
            y.min(ground)
        } else {
            y
        };

        (x as f32, y as f32)
    }

    /// Velocity (vx, vy) at time t.
    pub fn velocity_at(&self, t: SignalTime) -> (f32, f32) {
        let t = finite_or_f64(t, 0.0).max(0.0);

        let v0_x = finite_or(self.v0_x, 0.0) as f64;
        let v0_y = finite_or(self.v0_y, 0.0) as f64;
        let g = finite_or(self.gravity, 0.0) as f64;

        // vx is constant, vy increases with gravity
        let vx = v0_x;
        let vy = v0_y + g * t;

        (vx as f32, vy as f32)
    }

    /// Time when projectile reaches ground_y, if specified.
    /// Returns None if no ground or if starting below ground.
    pub fn time_to_ground(&self) -> Option<f64> {
        let ground_y = self.ground_y?;
        let ground = finite_or(ground_y, 0.0) as f64;
        let start_y = finite_or(self.start_y, 0.0) as f64;
        let v0_y = finite_or(self.v0_y, 0.0) as f64;
        let g = finite_or(self.gravity, 0.0) as f64;

        if g.abs() < 1e-10 {
            // No gravity - check if moving toward ground
            if v0_y.abs() < 1e-10 {
                return None; // Stationary
            }
            let t = (ground - start_y) / v0_y;
            return if t > 0.0 { Some(t) } else { None };
        }

        // Solve: start_y + v0_y*t + 0.5*g*t^2 = ground_y
        // => 0.5*g*t^2 + v0_y*t + (start_y - ground_y) = 0
        let a = 0.5 * g;
        let b = v0_y;
        let c = start_y - ground;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None; // Never reaches ground
        }

        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b + sqrt_disc) / (2.0 * a);
        let t2 = (-b - sqrt_disc) / (2.0 * a);

        // Return the smallest positive time (no Vec allocation)
        const EPS: f64 = 1e-10;
        let t1_valid = t1 > EPS;
        let t2_valid = t2 > EPS;

        match (t1_valid, t2_valid) {
            (true, true) => Some(t1.min(t2)),
            (true, false) => Some(t1),
            (false, true) => Some(t2),
            (false, false) => None,
        }
    }
}

impl Default for BallisticTrajectory {
    fn default() -> Self {
        Self {
            start_x: 0.0,
            start_y: 0.0,
            v0_x: 100.0,
            v0_y: -100.0, // Upward toss
            gravity: 500.0,
            ground_y: Some(300.0),
        }
    }
}

impl Signal for BallisticTrajectory {
    fn output_range(&self) -> SignalRange {
        // Range is from start_y to ground_y (or unbounded if no ground)
        let start_y = finite_or(self.start_y, 0.0);
        if let Some(ground) = self.ground_y {
            let ground = finite_or(ground, 0.0);
            let min = start_y.min(ground);
            let max = start_y.max(ground);
            SignalRange::new(min, max)
        } else {
            // No ground - estimate based on initial conditions
            SignalRange::new(start_y - 200.0, start_y + 200.0)
        }
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let (_, y) = self.position_at(t);
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.01;

    #[test]
    fn test_position_at_zero() {
        let traj = BallisticTrajectory::new(10.0, 20.0, 5.0, -10.0, 100.0, None);
        let (x, y) = traj.position_at(0.0);
        assert!((x - 10.0).abs() < EPSILON);
        assert!((y - 20.0).abs() < EPSILON);
    }

    #[test]
    fn test_horizontal_motion() {
        let traj = BallisticTrajectory::new(0.0, 0.0, 100.0, 0.0, 0.0, None);
        let (x, y) = traj.position_at(1.0);
        assert!((x - 100.0).abs() < EPSILON);
        assert!(y.abs() < EPSILON);
    }

    #[test]
    fn test_free_fall() {
        let traj = BallisticTrajectory::new(0.0, 0.0, 0.0, 0.0, 100.0, None);
        // y = 0.5 * g * t^2 = 0.5 * 100 * 1 = 50
        let (_, y) = traj.position_at(1.0);
        assert!((y - 50.0).abs() < EPSILON);
    }

    #[test]
    fn test_upward_toss() {
        let traj = BallisticTrajectory::new(0.0, 0.0, 0.0, -100.0, 100.0, None);
        // Goes up initially
        let (_, y_early) = traj.position_at(0.5);
        assert!(y_early < 0.0, "Should be above start");

        // Comes back down
        let (_, y_late) = traj.position_at(3.0);
        assert!(y_late > 0.0, "Should be below start");
    }

    #[test]
    fn test_ground_collision() {
        let traj = BallisticTrajectory::drop_from(0.0, 100.0, 500.0);
        // After enough time, should be clamped to ground
        let (_, y) = traj.position_at(10.0);
        assert!((y - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_time_to_ground() {
        let traj = BallisticTrajectory::drop_from(0.0, 100.0, 200.0);
        // y = 0.5 * 200 * t^2 = 100 => t = 1
        let t = traj.time_to_ground();
        assert!(t.is_some());
        assert!((t.unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_velocity_at() {
        let traj = BallisticTrajectory::new(0.0, 0.0, 10.0, -50.0, 100.0, None);
        let (vx, vy) = traj.velocity_at(0.0);
        assert!((vx - 10.0).abs() < EPSILON);
        assert!((vy - (-50.0)).abs() < EPSILON);

        // After 1 second, vy = -50 + 100 = 50
        let (_, vy_1) = traj.velocity_at(1.0);
        assert!((vy_1 - 50.0).abs() < EPSILON);
    }

    #[test]
    fn test_signal_sample() {
        let traj = BallisticTrajectory::default();
        let y = traj.sample(0.0);
        assert!((y - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_nan_handling() {
        let traj = BallisticTrajectory::new(f32::NAN, 0.0, 10.0, 0.0, 100.0, None);
        let (x, _) = traj.position_at(1.0);
        assert!(x.is_finite());
    }
}

// <FILE>src/physics/cls_projectile.rs</FILE> - <DESC>Ballistic trajectory solver</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
