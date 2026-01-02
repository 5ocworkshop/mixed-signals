// <FILE>mixed-signals/src/envelopes/cls_linear.rs</FILE> - <DESC>Simple linear envelope</DESC>
// <VERS>VERSION: 1.1.0 - 2025-12-27</VERS>
// <WCTX>Migrate time types from f32 to f64</WCTX>
// <CLOG>Updated Signal::sample parameter from f32 to f64</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalTime};
use serde::{Deserialize, Serialize};

/// Simple linear attack-release envelope.
///
/// Ramps up linearly during attack, holds at peak, then ramps down during release.
/// Simpler than ADSR when you don't need decay/sustain distinction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinearEnvelope {
    /// Attack time as fraction of total (0..1)
    pub attack: f32,
    /// Release time as fraction of total (0..1)
    pub release: f32,
    /// Peak level (default 1.0)
    pub peak: f32,
}

impl LinearEnvelope {
    pub fn new(attack: f32, release: f32) -> Self {
        let mut attack = attack.clamp(0.0, 1.0);
        let mut release = release.clamp(0.0, 1.0);
        let total = attack + release;
        if total > 1.0 {
            let scale = 1.0 / total;
            attack *= scale;
            release *= scale;
        }

        Self {
            attack,
            release,
            peak: 1.0,
        }
    }

    pub fn with_peak(mut self, peak: f32) -> Self {
        self.peak = peak;
        self
    }

    /// Symmetric attack and release
    pub fn symmetric(time: f32) -> Self {
        Self::new(time, time)
    }
}

impl Default for LinearEnvelope {
    fn default() -> Self {
        Self {
            attack: 0.1,
            release: 0.1,
            peak: 1.0,
        }
    }
}

impl Signal for LinearEnvelope {
    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).clamp(0.0, 1.0);
        let attack = finite_or(self.attack, 0.1) as f64;
        let release = finite_or(self.release, 0.1) as f64;
        let peak = finite_or(self.peak, 1.0) as f64;

        let hold_start = attack;
        let hold_end = 1.0 - release;

        let value = if t < hold_start {
            // Attack phase
            if attack > 0.0 {
                (t / attack) * peak
            } else {
                peak
            }
        } else if t < hold_end {
            // Hold phase
            peak
        } else {
            // Release phase
            if release > 0.0 {
                let release_progress = (t - hold_end) / release;
                peak * (1.0 - release_progress)
            } else {
                0.0
            }
        };

        value.clamp(0.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_attack() {
        let env = LinearEnvelope::new(0.2, 0.2);
        assert!((env.sample(0.0) - 0.0).abs() < 0.001);
        assert!((env.sample(0.1) - 0.5).abs() < 0.001);
        assert!((env.sample(0.2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_hold() {
        let env = LinearEnvelope::new(0.2, 0.2);
        assert!((env.sample(0.3) - 1.0).abs() < 0.001);
        assert!((env.sample(0.5) - 1.0).abs() < 0.001);
        assert!((env.sample(0.7) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_release() {
        let env = LinearEnvelope::new(0.2, 0.2);
        assert!((env.sample(0.9) - 0.5).abs() < 0.001);
        assert!((env.sample(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_symmetric() {
        let env = LinearEnvelope::symmetric(0.25);
        assert!((env.sample(0.0) - 0.0).abs() < 0.001);
        assert!((env.sample(0.25) - 1.0).abs() < 0.001);
        assert!((env.sample(0.5) - 1.0).abs() < 0.001);
        assert!((env.sample(0.75) - 1.0).abs() < 0.001);
        assert!((env.sample(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_normalizes_phase_sum() {
        let env = LinearEnvelope::new(0.8, 0.6);
        let total = env.attack + env.release;
        assert!(total <= 1.0 + 1e-6, "phase sum not normalized: {}", total);
    }
}

// <FILE>mixed-signals/src/envelopes/cls_linear.rs</FILE> - <DESC>Simple linear envelope</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-27</VERS>
