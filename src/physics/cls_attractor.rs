// <FILE>src/physics/cls_attractor.rs</FILE> - <DESC>Point attractor force field</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Physics module implementation</WCTX>
// <CLOG>Initial implementation with inverse-square force field</CLOG>

use crate::math::finite_or;
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Point attractor force field.
///
/// Models an attractive (or repulsive) force toward a point.
/// Useful for snap-to-point effects, magnetic UI elements, or bubble effects.
///
/// # Physics Model
///
/// Force magnitude follows inverse-square law:
/// `F = strength / (distance² + softening)`
///
/// The softening factor prevents singularity at distance=0.
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the force magnitude at distance `t`.
/// This treats `t` as the distance from the attractor center.
///
/// For actual particle simulation, use `force_at()` which returns a force vector.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::PointAttractor;
///
/// let attractor = PointAttractor::new(100.0, 100.0, 500.0);
/// let (fx, fy) = attractor.force_at(150.0, 100.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PointAttractor {
    /// Target X coordinate (attractor center).
    pub target_x: f32,
    /// Target Y coordinate (attractor center).
    pub target_y: f32,
    /// Attraction strength. Positive = attractive, negative = repulsive.
    pub strength: f32,
}

impl PointAttractor {
    /// Create a new point attractor.
    pub fn new(target_x: f32, target_y: f32, strength: f32) -> Self {
        Self {
            target_x,
            target_y,
            strength,
        }
    }

    /// Attractor at origin with given strength.
    pub fn at_origin(strength: f32) -> Self {
        Self::new(0.0, 0.0, strength)
    }

    /// Calculate force vector on a particle at (px, py).
    ///
    /// Returns (fx, fy) - the force components pointing toward the attractor.
    pub fn force_at(&self, px: f32, py: f32) -> (f32, f32) {
        let tx = finite_or(self.target_x, 0.0);
        let ty = finite_or(self.target_y, 0.0);
        let strength = finite_or(self.strength, 0.0);
        let px = finite_or(px, 0.0);
        let py = finite_or(py, 0.0);

        // Direction toward target
        let dx = tx - px;
        let dy = ty - py;

        // Distance with softening to prevent singularity
        let dist_sq = dx * dx + dy * dy;
        let softening = 1.0; // Prevents division by zero
        let dist = (dist_sq + softening).sqrt();

        if dist < 1e-6 {
            return (0.0, 0.0);
        }

        // Inverse-square force magnitude
        let force_magnitude = strength / (dist_sq + softening);

        // Normalize direction and apply magnitude
        let fx = force_magnitude * dx / dist;
        let fy = force_magnitude * dy / dist;

        (fx, fy)
    }

    /// Force magnitude at a given distance.
    pub fn force_magnitude_at(&self, distance: f32) -> f32 {
        let distance = finite_or(distance, 0.0).abs();
        let strength = finite_or(self.strength, 0.0).abs();
        let softening = 1.0;

        strength / (distance * distance + softening)
    }

    /// Distance from attractor center to a point.
    pub fn distance_to(&self, px: f32, py: f32) -> f32 {
        let tx = finite_or(self.target_x, 0.0);
        let ty = finite_or(self.target_y, 0.0);
        let dx = tx - px;
        let dy = ty - py;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for PointAttractor {
    fn default() -> Self {
        Self {
            target_x: 0.0,
            target_y: 0.0,
            strength: 100.0,
        }
    }
}

impl Signal for PointAttractor {
    fn output_range(&self) -> SignalRange {
        // Force magnitude ranges from 0 to strength (at zero distance)
        let max_force = finite_or(self.strength, 100.0).abs();
        SignalRange::new(0.0, max_force)
    }

    /// Treats t as distance, returns force magnitude.
    fn sample(&self, t: SignalTime) -> f32 {
        self.force_magnitude_at(t as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.01;

    #[test]
    fn test_force_at_origin() {
        let attractor = PointAttractor::at_origin(100.0);
        // Particle at (10, 0), force should point toward origin
        let (fx, fy) = attractor.force_at(10.0, 0.0);
        assert!(fx < 0.0, "Force should point toward origin (negative x)");
        assert!(fy.abs() < EPSILON, "No y component when on x-axis");
    }

    #[test]
    fn test_force_direction() {
        let attractor = PointAttractor::new(100.0, 100.0, 50.0);
        // Particle at (50, 100), force should point right (+x)
        let (fx, _) = attractor.force_at(50.0, 100.0);
        assert!(fx > 0.0, "Force should point toward target");
    }

    #[test]
    fn test_repulsive_force() {
        let attractor = PointAttractor::at_origin(-100.0);
        let (fx, _) = attractor.force_at(10.0, 0.0);
        assert!(fx > 0.0, "Negative strength should repel (positive x)");
    }

    #[test]
    fn test_force_decreases_with_distance() {
        let attractor = PointAttractor::at_origin(100.0);
        let (fx_near, _) = attractor.force_at(5.0, 0.0);
        let (fx_far, _) = attractor.force_at(20.0, 0.0);

        assert!(
            fx_near.abs() > fx_far.abs(),
            "Force should be stronger when closer"
        );
    }

    #[test]
    fn test_force_at_target() {
        let attractor = PointAttractor::at_origin(100.0);
        // Particle exactly at attractor center
        let (fx, fy) = attractor.force_at(0.0, 0.0);
        // Should be zero (softening handles singularity)
        assert!(fx.abs() < EPSILON);
        assert!(fy.abs() < EPSILON);
    }

    #[test]
    fn test_force_magnitude_at() {
        let attractor = PointAttractor::at_origin(100.0);
        let f_near = attractor.force_magnitude_at(1.0);
        let f_far = attractor.force_magnitude_at(10.0);
        assert!(f_near > f_far);
    }

    #[test]
    fn test_distance_to() {
        let attractor = PointAttractor::new(10.0, 0.0, 100.0);
        let dist = attractor.distance_to(0.0, 0.0);
        assert!((dist - 10.0).abs() < EPSILON);
    }

    #[test]
    fn test_signal_sample() {
        let attractor = PointAttractor::at_origin(100.0);
        let force_at_5 = attractor.sample(5.0);
        let force_at_10 = attractor.sample(10.0);
        assert!(force_at_5 > force_at_10);
    }

    #[test]
    fn test_output_range() {
        let attractor = PointAttractor::at_origin(200.0);
        let range = attractor.output_range();
        assert!((range.min - 0.0).abs() < EPSILON);
        assert!((range.max - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_nan_handling() {
        let attractor = PointAttractor::new(f32::NAN, 0.0, 100.0);
        let (fx, fy) = attractor.force_at(10.0, 0.0);
        assert!(fx.is_finite());
        assert!(fy.is_finite());
    }
}

// <FILE>src/physics/cls_attractor.rs</FILE> - <DESC>Point attractor force field</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
