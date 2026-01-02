// <FILE>mixed-signals/src/envelopes/cls_impact.rs</FILE> - <DESC>Impact/decay envelope</DESC>
// <VERS>VERSION: 1.1.0 - 2025-12-27</VERS>
// <WCTX>Migrate time types from f32 to f64</WCTX>
// <CLOG>Updated Signal::sample parameter from f32 to f64</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalTime};
use serde::{Deserialize, Serialize};

/// Impact envelope with instant attack and exponential decay.
///
/// Useful for shake effects, hit feedback, or any sudden impact
/// that fades over time.
///
/// Formula: `output = intensity * e^(-decay * t)`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Impact {
    /// Starting intensity (peak value at t=0)
    pub intensity: f32,
    /// Decay rate (higher = faster decay)
    pub decay: f32,
}

impl Impact {
    pub fn new(intensity: f32, decay: f32) -> Self {
        Self {
            intensity,
            decay: decay.max(0.0),
        }
    }

    /// Create with default decay rate
    pub fn with_intensity(intensity: f32) -> Self {
        Self::new(intensity, 3.0)
    }
}

impl Default for Impact {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            decay: 3.0,
        }
    }
}

impl Signal for Impact {
    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0);
        let intensity = finite_or(self.intensity, 1.0) as f64;
        let decay = finite_or(self.decay, 3.0) as f64;

        if t < 0.0 {
            return intensity.clamp(0.0, 1.0) as f32;
        }
        (intensity * (-decay * t).exp()).clamp(0.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_at_zero() {
        let impact = Impact::default();
        assert!((impact.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_impact_decays() {
        let impact = Impact::default();
        let v0 = impact.sample(0.0);
        let v1 = impact.sample(0.5);
        let v2 = impact.sample(1.0);
        assert!(v0 > v1);
        assert!(v1 > v2);
        assert!(v2 > 0.0);
    }

    #[test]
    fn test_impact_decay_rate() {
        let slow = Impact::new(1.0, 1.0);
        let fast = Impact::new(1.0, 5.0);
        // Fast decay should be lower at same time
        assert!(fast.sample(0.5) < slow.sample(0.5));
    }

    #[test]
    fn test_impact_intensity() {
        let impact = Impact::new(2.0, 3.0);
        assert!((impact.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_impact_approaches_zero() {
        let impact = Impact::default();
        // At t=3 with decay=3, should be e^-9 ≈ 0.0001
        assert!(impact.sample(3.0) < 0.01);
    }
}

// <FILE>mixed-signals/src/envelopes/cls_impact.rs</FILE> - <DESC>Impact/decay envelope</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-27</VERS>
