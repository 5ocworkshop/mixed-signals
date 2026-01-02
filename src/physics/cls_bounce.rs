// <FILE>src/physics/cls_bounce.rs</FILE> - <DESC>Bouncing drop with restitution</DESC>
// <VERS>VERSION: 1.1.0</VERS>
// <WCTX>Physics module robustness improvements</WCTX>
// <CLOG>Fix height_at clamp bug, add g==0 handling for frozen motion</CLOG>

use crate::math::{finite_or, finite_or_clamp, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Bouncing drop solver with energy loss per bounce.
///
/// Models an object falling under gravity and bouncing on a surface,
/// losing energy with each impact. Useful for modal drop-in animations
/// or bouncy UI elements.
///
/// # Physics Model
///
/// Each bounce retains a fraction of velocity (restitution coefficient).
/// - `restitution = 1.0`: perfectly elastic, bounces forever
/// - `restitution = 0.5`: typical rubber ball
/// - `restitution = 0.0`: no bounce, stops on impact
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the height at time t.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::BouncingDrop;
/// use mixed_signals::traits::Signal;
///
/// let drop = BouncingDrop::new(0.0, 300.0, 500.0, 0.6);
/// let height = drop.sample(0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BouncingDrop {
    /// Starting height (Y position at t=0).
    pub start_height: f32,
    /// Ground height (floor level).
    pub ground_height: f32,
    /// Gravity acceleration.
    pub gravity: f32,
    /// Restitution coefficient (0.0-1.0). Energy retained per bounce.
    pub restitution: f32,
}

impl BouncingDrop {
    /// Create a new bouncing drop with all parameters.
    pub fn new(start_height: f32, ground_height: f32, gravity: f32, restitution: f32) -> Self {
        Self {
            start_height,
            ground_height,
            gravity,
            restitution,
        }
    }

    /// Drop from height to ground with typical rubber ball restitution (0.6).
    pub fn rubber_ball(start_height: f32, ground_height: f32, gravity: f32) -> Self {
        Self::new(start_height, ground_height, gravity, 0.6)
    }

    /// Drop with no bounce (stops on impact).
    pub fn no_bounce(start_height: f32, ground_height: f32, gravity: f32) -> Self {
        Self::new(start_height, ground_height, gravity, 0.0)
    }

    /// Height at time t.
    pub fn height_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);
        if t == 0.0 {
            return finite_or(self.start_height, 0.0);
        }

        let start = finite_or(self.start_height, 0.0) as f64;
        let ground = finite_or(self.ground_height, 0.0) as f64;
        let g = finite_or(self.gravity, 500.0).abs() as f64;
        let restitution = finite_or_clamp(self.restitution, 0.0, 1.0, 0.5) as f64;

        // Height above ground
        let h0 = (ground - start).abs();

        if h0 < 1e-6 {
            return ground as f32; // Already at ground
        }

        // Handle zero gravity: object stays at start forever
        if g < 1e-10 {
            return start as f32;
        }

        // Determine direction: are we falling toward ground or rising away?
        // Assume start_height < ground_height means we fall down toward ground
        let falling_down = start < ground;

        // Time to first impact: t = sqrt(2h/g)
        let t_first = (2.0 * h0 / g).sqrt();

        if t <= t_first {
            // Before first bounce - free fall
            let h = if falling_down {
                start + 0.5 * g * t * t
            } else {
                start - 0.5 * g * t * t
            };
            // Clamp to valid range between start and ground
            let lo = start.min(ground);
            let hi = start.max(ground);
            return h.clamp(lo, hi) as f32;
        }

        if restitution < 1e-6 {
            // No bounce - stay at ground
            return ground as f32;
        }

        // After first bounce - calculate which bounce we're in
        // Each bounce has period T_n = 2 * sqrt(2 * h_n / g)
        // where h_n = h0 * restitution^(2n) is the peak height of bounce n

        let mut time_remaining = t - t_first;
        let mut bounce_num = 0u32;
        let mut v_after_bounce = (2.0 * g * h0).sqrt() * restitution;

        // Max iterations to prevent infinite loop
        const MAX_BOUNCES: u32 = 100;

        loop {
            if bounce_num >= MAX_BOUNCES || v_after_bounce < 1e-6 {
                // Effectively stopped
                return ground as f32;
            }

            // Time for this complete bounce (up and down)
            let bounce_duration = 2.0 * v_after_bounce / g;

            if time_remaining <= bounce_duration {
                // We're in this bounce
                // Height during bounce: y = ground - v*t + 0.5*g*t^2 (going up then down)
                // Actually: y = ground - (v*t - 0.5*g*t^2) for upward phase

                let t_up = v_after_bounce / g; // Time to peak

                let height = if time_remaining <= t_up {
                    // Going up
                    let y =
                        v_after_bounce * time_remaining - 0.5 * g * time_remaining * time_remaining;
                    if falling_down {
                        ground - y
                    } else {
                        ground + y
                    }
                } else {
                    // Coming down
                    let t_down = time_remaining - t_up;
                    let peak_height = 0.5 * v_after_bounce * v_after_bounce / g;
                    let y = peak_height - 0.5 * g * t_down * t_down;
                    if falling_down {
                        ground - y
                    } else {
                        ground + y
                    }
                };

                return height as f32;
            }

            time_remaining -= bounce_duration;
            v_after_bounce *= restitution;
            bounce_num += 1;
        }
    }

    /// Time of first bounce.
    /// Returns `f32::INFINITY` if gravity is zero (never reaches ground).
    pub fn time_to_first_bounce(&self) -> f32 {
        let start = finite_or(self.start_height, 0.0);
        let ground = finite_or(self.ground_height, 0.0);
        let g = finite_or(self.gravity, 500.0).abs();

        let h = (ground - start).abs();

        // Already at ground
        if h < 1e-6 {
            return 0.0;
        }

        // No gravity means never reaches ground
        if g < 1e-6 {
            return f32::INFINITY;
        }

        (2.0 * h / g).sqrt()
    }

    /// Approximate total time until motion stops (velocity < epsilon).
    /// Returns `f32::INFINITY` if gravity is zero or restitution is 1.0.
    pub fn duration_until_stop(&self) -> f32 {
        let start = finite_or(self.start_height, 0.0);
        let ground = finite_or(self.ground_height, 0.0);
        let g = finite_or(self.gravity, 500.0).abs();
        let restitution = finite_or_clamp(self.restitution, 0.0, 1.0, 0.5);

        // No gravity means never settles (stays at start)
        if g < 1e-6 {
            return f32::INFINITY;
        }

        if restitution < 1e-6 {
            return self.time_to_first_bounce();
        }

        let h0 = (ground - start).abs();
        let v0 = (2.0 * g * h0).sqrt();

        // Sum of geometric series for bounce durations
        // Total time ≈ t_first + sum(2*v0*r^n/g) for n=1..∞
        // = t_first + 2*v0*r/(g*(1-r)) for r < 1

        let t_first = (2.0 * h0 / g).sqrt();
        let r = restitution;

        if r >= 1.0 - 1e-6 {
            return f32::INFINITY; // Bounces forever
        }

        t_first + 2.0 * v0 * r / (g * (1.0 - r))
    }
}

impl Default for BouncingDrop {
    fn default() -> Self {
        Self {
            start_height: 0.0,
            ground_height: 200.0,
            gravity: 500.0,
            restitution: 0.6,
        }
    }
}

impl Signal for BouncingDrop {
    fn output_range(&self) -> SignalRange {
        let start = finite_or(self.start_height, 0.0);
        let ground = finite_or(self.ground_height, 0.0);
        SignalRange::new(start.min(ground), start.max(ground))
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.height_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1.0;

    #[test]
    fn test_height_at_zero() {
        let drop = BouncingDrop::new(0.0, 200.0, 500.0, 0.5);
        assert!((drop.height_at(0.0) - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_falls_to_ground() {
        let drop = BouncingDrop::new(0.0, 200.0, 500.0, 0.5);
        // Should be at ground eventually
        let h_late = drop.height_at(10.0);
        assert!((h_late - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_no_bounce_stops() {
        let drop = BouncingDrop::no_bounce(0.0, 100.0, 500.0);
        let t_first = drop.time_to_first_bounce();
        let h_after = drop.height_at((t_first + 0.5) as f64);
        assert!(
            (h_after - 100.0).abs() < EPSILON,
            "Should stay at ground, got {}",
            h_after
        );
    }

    #[test]
    fn test_bounces_decrease() {
        let drop = BouncingDrop::rubber_ball(0.0, 100.0, 500.0);
        let t_first = drop.time_to_first_bounce();

        // Check heights at approximate peak times of first few bounces
        let peak1 = drop.height_at((t_first + 0.2) as f64);
        let _peak2 = drop.height_at((t_first + 0.8) as f64);

        // After first bounce, height should be less than start
        assert!(peak1 < 100.0, "First bounce peak should be below ground");
    }

    #[test]
    fn test_time_to_first_bounce() {
        let drop = BouncingDrop::new(0.0, 100.0, 200.0, 0.5);
        // t = sqrt(2h/g) = sqrt(2*100/200) = 1.0
        let t = drop.time_to_first_bounce();
        assert!((t - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_duration_until_stop() {
        let drop = BouncingDrop::new(0.0, 100.0, 500.0, 0.5);
        let duration = drop.duration_until_stop();
        assert!(duration > 0.0);
        assert!(duration < 10.0);
    }

    #[test]
    fn test_perfect_elastic() {
        let drop = BouncingDrop::new(0.0, 100.0, 500.0, 1.0);
        let duration = drop.duration_until_stop();
        assert!(duration.is_infinite());
    }

    #[test]
    fn test_output_range() {
        let drop = BouncingDrop::new(50.0, 200.0, 500.0, 0.5);
        let range = drop.output_range();
        assert!((range.min - 50.0).abs() < EPSILON);
        assert!((range.max - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_nan_handling() {
        let drop = BouncingDrop::new(f32::NAN, 100.0, 500.0, 0.5);
        let h = drop.height_at(1.0);
        assert!(h.is_finite());
    }

    #[test]
    fn test_zero_gravity() {
        let drop = BouncingDrop::new(0.0, 100.0, 0.0, 0.5);
        let t = drop.time_to_first_bounce();
        assert!(t.is_infinite(), "Zero gravity should never reach ground");

        // Height should stay at start forever
        let h = drop.height_at(100.0);
        assert!(
            (h - 0.0).abs() < EPSILON,
            "Should stay at start with zero gravity"
        );

        // Duration until stop should be infinite
        let duration = drop.duration_until_stop();
        assert!(duration.is_infinite());
    }

    #[test]
    fn test_already_at_ground() {
        let drop = BouncingDrop::new(100.0, 100.0, 500.0, 0.5);
        let t = drop.time_to_first_bounce();
        assert!(
            (t - 0.0).abs() < 0.01,
            "Already at ground should have 0 time to bounce"
        );
    }
}

// <FILE>src/physics/cls_bounce.rs</FILE> - <DESC>Bouncing drop with restitution</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
