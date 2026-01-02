// <FILE>mixed-signals/src/envelopes/cls_adsr.rs</FILE> - <DESC>ADSR envelope generator</DESC>
// <VERS>VERSION: 1.1.0 - 2025-12-27</VERS>
// <WCTX>Migrate time types from f32 to f64</WCTX>
// <CLOG>Updated Signal::sample parameter from f32 to f64</CLOG>

use crate::math::{finite_or, finite_or_f64};
use crate::traits::{Signal, SignalTime};
use serde::{Deserialize, Serialize};

/// ADSR (Attack-Decay-Sustain-Release) envelope generator.
///
/// Classic synthesizer envelope shape:
/// - Attack: Ramp from 0 to peak
/// - Decay: Ramp from peak to sustain level
/// - Sustain: Hold at sustain level
/// - Release: Ramp from sustain to 0
///
/// Time is treated as normalized progress (0..1) over the envelope duration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Adsr {
    /// Attack time as fraction of total (0..1)
    pub attack: f32,
    /// Decay time as fraction of total (0..1)
    pub decay: f32,
    /// Sustain level (0..1)
    pub sustain: f32,
    /// Release time as fraction of total (0..1)
    pub release: f32,
    /// Peak level at end of attack (default 1.0)
    pub peak: f32,
}

impl Adsr {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let mut attack = attack.clamp(0.0, 1.0);
        let mut decay = decay.clamp(0.0, 1.0);
        let mut release = release.clamp(0.0, 1.0);
        let sustain = sustain.clamp(0.0, 1.0);

        let total = attack + decay + release;
        if total > 1.0 {
            let scale = 1.0 / total;
            attack *= scale;
            decay *= scale;
            release *= scale;
        }

        Self {
            attack,
            decay,
            sustain,
            release,
            peak: 1.0,
        }
    }

    pub fn with_peak(mut self, peak: f32) -> Self {
        self.peak = peak;
        self
    }
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack: 0.1,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            peak: 1.0,
        }
    }
}

impl Signal for Adsr {
    fn sample(&self, t: SignalTime) -> f32 {
        let t = finite_or_f64(t, 0.0).clamp(0.0, 1.0);

        let attack = finite_or(self.attack, 0.1) as f64;
        let decay = finite_or(self.decay, 0.1) as f64;
        let sustain = finite_or(self.sustain, 0.7) as f64;
        let release = finite_or(self.release, 0.2) as f64;
        let peak = finite_or(self.peak, 1.0) as f64;

        // Calculate phase boundaries
        let attack_end = attack;
        let decay_end = attack_end + decay;
        let sustain_end = 1.0 - release;

        let value = if t < attack_end {
            // Attack phase: 0 -> peak
            if attack > 0.0 {
                (t / attack) * peak
            } else {
                peak
            }
        } else if t < decay_end {
            // Decay phase: peak -> sustain
            if decay > 0.0 {
                let decay_progress = (t - attack_end) / decay;
                peak - (peak - sustain * peak) * decay_progress
            } else {
                sustain * peak
            }
        } else if t < sustain_end {
            // Sustain phase: hold at sustain level
            sustain * peak
        } else {
            // Release phase: sustain -> 0
            if release > 0.0 {
                let release_progress = (t - sustain_end) / release;
                sustain * peak * (1.0 - release_progress)
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
    fn test_adsr_attack() {
        let env = Adsr::new(0.2, 0.2, 0.5, 0.2);
        assert!((env.sample(0.0) - 0.0).abs() < 0.001);
        assert!((env.sample(0.1) - 0.5).abs() < 0.001); // Halfway through attack
        assert!((env.sample(0.2) - 1.0).abs() < 0.001); // End of attack
    }

    #[test]
    fn test_adsr_decay() {
        let env = Adsr::new(0.2, 0.2, 0.5, 0.2);
        // At t=0.3 (halfway through decay), should be between 1.0 and 0.5
        let v = env.sample(0.3);
        assert!(v > 0.5 && v < 1.0, "Value {} not in decay range", v);
    }

    #[test]
    fn test_adsr_sustain() {
        let env = Adsr::new(0.2, 0.2, 0.5, 0.2);
        // At t=0.5 (sustain phase), should be at sustain level
        assert!((env.sample(0.5) - 0.5).abs() < 0.001);
        assert!((env.sample(0.7) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_adsr_release() {
        let env = Adsr::new(0.2, 0.2, 0.5, 0.2);
        // At t=1.0, should be at 0
        assert!((env.sample(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_adsr_normalizes_phase_sum() {
        let env = Adsr::new(0.6, 0.6, 0.5, 0.6);
        let total = env.attack + env.decay + env.release;
        assert!(total <= 1.0 + 1e-6, "phase sum not normalized: {}", total);
    }
}

// <FILE>mixed-signals/src/envelopes/cls_adsr.rs</FILE> - <DESC>ADSR envelope generator</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-27</VERS>
