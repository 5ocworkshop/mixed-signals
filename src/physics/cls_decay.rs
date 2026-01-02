// <FILE>src/physics/cls_decay.rs</FILE> - <DESC>Friction/inertia decay solver</DESC>
// <VERS>VERSION: 1.1.0</VERS>
// <WCTX>Physics module robustness improvements</WCTX>
// <CLOG>Add exp_m1 for precision, explicit drag==0 branch for linear motion</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Friction/inertia decay solver for scrolling and flinging.
///
/// Models exponential velocity decay with drag, producing smooth deceleration.
/// Useful for scroll momentum, fling gestures, or sliding elements.
///
/// # Physics Model
///
/// Uses exponential decay: `v(t) = v0 * e^(-drag * t)`
///
/// Total displacement: `offset(t) = (v0 / drag) * (1 - e^(-drag * t))`
///
/// As t → ∞, offset approaches `v0 / drag` (maximum travel distance).
///
/// # Signal Integration
///
/// Implements `Signal` where `sample(t)` returns the displacement from start.
///
/// # Example
///
/// ```rust
/// use mixed_signals::physics::FrictionDecay;
/// use mixed_signals::traits::Signal;
///
/// let scroll = FrictionDecay::new(500.0, 3.0); // Fast flick, moderate drag
/// let offset = scroll.sample(0.5); // Displacement after 0.5s
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrictionDecay {
    /// Initial velocity (units/sec).
    pub v0: f32,
    /// Drag coefficient (exponential decay rate). Higher = faster stop.
    pub drag: f32,
}

impl FrictionDecay {
    /// Create a new friction decay with initial velocity and drag.
    pub fn new(v0: f32, drag: f32) -> Self {
        Self { v0, drag }
    }

    /// Light friction for smooth, long scrolling.
    pub fn light(v0: f32) -> Self {
        Self::new(v0, 2.0)
    }

    /// Heavy friction for quick stopping.
    pub fn heavy(v0: f32) -> Self {
        Self::new(v0, 8.0)
    }

    /// Total displacement at time t.
    pub fn offset_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);
        if t == 0.0 {
            return 0.0;
        }

        let v0 = finite_or(self.v0, 0.0) as f64;
        let drag = finite_or(self.drag, 0.0).max(0.0) as f64;

        // Zero drag = linear motion (no friction)
        if drag < 1e-10 {
            return (v0 * t) as f32;
        }

        // offset(t) = (v0 / drag) * (1 - e^(-drag * t))
        // Use exp_m1 for precision when drag*t is small:
        // (1 - e^(-x)) = -expm1(-x)
        let result = (v0 / drag) * (-(-drag * t).exp_m1());
        result as f32
    }

    /// Current velocity at time t.
    pub fn velocity_at(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).max(0.0);

        let v0 = finite_or(self.v0, 0.0) as f64;
        let drag = finite_or(self.drag, 0.0).max(0.0) as f64;

        // Zero drag = constant velocity (no friction)
        if drag < 1e-10 {
            return v0 as f32;
        }

        // v(t) = v0 * e^(-drag * t)
        let result = v0 * (-drag * t).exp();
        result as f32
    }

    /// Time until velocity drops below epsilon.
    /// Returns `f32::INFINITY` if drag is zero (never stops).
    pub fn duration_until_stop(&self, epsilon: f32) -> f32 {
        let v0 = finite_or(self.v0, 0.0).abs();
        let epsilon = epsilon.abs().max(0.001);
        let drag = finite_or(self.drag, 0.0).max(0.0);

        if v0 <= epsilon {
            return 0.0;
        }

        // Zero drag = never stops
        if drag < 1e-10 {
            return f32::INFINITY;
        }

        // Solve: v0 * e^(-drag * t) = epsilon
        // => t = -ln(epsilon / v0) / drag
        (v0 / epsilon).ln() / drag
    }

    /// Maximum displacement (as t → ∞).
    /// Returns `f32::INFINITY` if drag is zero (linear motion forever).
    pub fn max_offset(&self) -> f32 {
        let v0 = finite_or(self.v0, 0.0);
        let drag = finite_or(self.drag, 0.0).max(0.0);

        // Zero drag = infinite displacement
        if drag < 1e-10 {
            return if v0 >= 0.0 {
                f32::INFINITY
            } else {
                f32::NEG_INFINITY
            };
        }

        v0 / drag
    }
}

impl Default for FrictionDecay {
    fn default() -> Self {
        Self {
            v0: 200.0,
            drag: 4.0,
        }
    }
}

impl Signal for FrictionDecay {
    fn output_range(&self) -> SignalRange {
        let max = self.max_offset();
        // Handle infinite case (zero drag)
        if max.is_infinite() {
            let v0 = finite_or(self.v0, 0.0);
            if v0 >= 0.0 {
                SignalRange::new(0.0, f32::MAX)
            } else {
                SignalRange::new(f32::MIN, 0.0)
            }
        } else if max >= 0.0 {
            SignalRange::new(0.0, max)
        } else {
            SignalRange::new(max, 0.0)
        }
    }

    fn sample(&self, t: SignalTime) -> f32 {
        self.offset_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.01;

    #[test]
    fn test_offset_at_zero() {
        let decay = FrictionDecay::new(100.0, 5.0);
        assert!(decay.offset_at(0.0).abs() < EPSILON);
    }

    #[test]
    fn test_velocity_at_zero() {
        let decay = FrictionDecay::new(100.0, 5.0);
        assert!((decay.velocity_at(0.0) - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_converges_to_max() {
        let decay = FrictionDecay::new(100.0, 5.0);
        let max = decay.max_offset(); // 100/5 = 20
        assert!((max - 20.0).abs() < EPSILON);

        // After long time, should be near max
        let offset = decay.offset_at(10.0);
        assert!((offset - max).abs() < 0.01);
    }

    #[test]
    fn test_velocity_decays() {
        let decay = FrictionDecay::new(100.0, 5.0);
        let v1 = decay.velocity_at(0.5);
        let v2 = decay.velocity_at(1.0);
        assert!(v1 < 100.0);
        assert!(v2 < v1);
    }

    #[test]
    fn test_high_drag_quick_stop() {
        let light = FrictionDecay::light(100.0);
        let heavy = FrictionDecay::heavy(100.0);

        let t_light = light.duration_until_stop(1.0);
        let t_heavy = heavy.duration_until_stop(1.0);

        assert!(t_heavy < t_light, "Heavy drag should stop faster");
    }

    #[test]
    fn test_negative_velocity() {
        let decay = FrictionDecay::new(-100.0, 5.0);
        let offset = decay.offset_at(1.0);
        assert!(
            offset < 0.0,
            "Negative velocity should produce negative offset"
        );
    }

    #[test]
    fn test_duration_until_stop() {
        let decay = FrictionDecay::new(100.0, 5.0);
        let t = decay.duration_until_stop(1.0);
        let v_at_t = decay.velocity_at(t as f64);
        assert!((v_at_t - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_output_range() {
        let decay = FrictionDecay::new(100.0, 4.0);
        let range = decay.output_range();
        assert!((range.min - 0.0).abs() < EPSILON);
        assert!((range.max - 25.0).abs() < EPSILON);
    }

    #[test]
    fn test_nan_handling() {
        let decay = FrictionDecay::new(f32::NAN, 5.0);
        let offset = decay.offset_at(1.0);
        assert!(offset.is_finite());
    }

    #[test]
    fn test_zero_drag_linear_motion() {
        let decay = FrictionDecay::new(100.0, 0.0);

        // Zero drag = linear motion: offset = v0 * t
        let offset = decay.offset_at(1.0);
        assert!(
            (offset - 100.0).abs() < EPSILON,
            "Expected linear motion: v0 * t"
        );

        // Velocity stays constant
        let v = decay.velocity_at(10.0);
        assert!((v - 100.0).abs() < EPSILON, "Velocity should stay constant");

        // Max offset is infinite
        let max = decay.max_offset();
        assert!(
            max.is_infinite(),
            "Max offset should be infinite with no drag"
        );

        // Duration until stop is infinite
        let duration = decay.duration_until_stop(1.0);
        assert!(duration.is_infinite(), "Should never stop with no drag");
    }
}

// <FILE>src/physics/cls_decay.rs</FILE> - <DESC>Friction/inertia decay solver</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
