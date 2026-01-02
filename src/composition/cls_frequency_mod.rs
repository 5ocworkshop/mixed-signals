// <FILE>mixed-signals/src/composition/cls_frequency_mod.rs</FILE> - <DESC>Frequency modulation operator</DESC>
// <VERS>VERSION: 1.0.0 - 2025-12-22</VERS>
// <WCTX>Signal generator implementation</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};
use std::f64::consts::TAU;

/// Phase modulation operator (commonly called "FM" in synthesizers).
///
/// Despite the name, this implements phase modulation rather than true
/// frequency modulation. This matches how most "FM" synthesizers actually
/// work (including the DX7). True FM would require integrating the modulator;
/// we modulate the phase directly for simplicity.
///
/// The modulator signal offsets the carrier's phase, creating evolving waveforms.
///
/// Output ≈ carrier(t + depth * modulator(t) / (TAU * carrier_freq))
#[derive(Debug, Clone)]
pub struct FrequencyMod<C, M> {
    pub carrier: C,
    pub modulator: M,
    /// Modulation depth (how much the modulator affects the carrier)
    pub depth: f32,
    /// Carrier frequency for the internal sine
    pub carrier_freq: f32,
}

impl<C: Signal, M: Signal> FrequencyMod<C, M> {
    pub fn new(carrier: C, modulator: M, depth: f32, carrier_freq: f32) -> Self {
        Self {
            carrier,
            modulator,
            depth,
            carrier_freq,
        }
    }

    /// Simple FM with default carrier frequency
    pub fn simple(carrier: C, modulator: M, depth: f32) -> Self {
        Self::new(carrier, modulator, depth, 1.0)
    }
}

impl<C: Signal, M: Signal> Signal for FrequencyMod<C, M> {
    fn sample(&self, t: SignalTime) -> f32 {
        if !self.carrier_freq.is_finite() || self.carrier_freq == 0.0 {
            return self.carrier.sample(t);
        }
        if !self.depth.is_finite() {
            return self.carrier.sample(t);
        }
        // Get modulator value and use it to offset the phase
        let mod_value = self.modulator.sample(t) * 2.0 - 1.0;
        let phase_offset = self.depth as f64 * mod_value as f64;

        // Phase modulation (not true FM, but standard in synthesizers)
        let modulated_t = t + phase_offset / (TAU * self.carrier_freq as f64);

        self.carrier.sample(modulated_t)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        if !self.carrier_freq.is_finite() || self.carrier_freq == 0.0 {
            return self.carrier.sample_with_context(t, ctx);
        }
        if !self.depth.is_finite() {
            return self.carrier.sample_with_context(t, ctx);
        }
        let mod_value = self.modulator.sample_with_context(t, ctx) * 2.0 - 1.0;
        let phase_offset = self.depth as f64 * mod_value as f64;
        let modulated_t = t + phase_offset / (TAU * self.carrier_freq as f64);
        self.carrier.sample_with_context(modulated_t, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_fm_zero_depth() {
        let carrier = Sine::with_frequency(1.0);
        let modulator = Sine::with_frequency(2.0);
        let fm = FrequencyMod::simple(carrier, modulator, 0.0);

        // With zero depth, should match original carrier
        assert!((fm.sample(0.25) - carrier.sample(0.25)).abs() < 0.001);
    }

    #[test]
    fn test_fm_with_constant_modulator() {
        let carrier = Sine::with_frequency(1.0);
        let modulator = Constant::new(0.5);
        let fm = FrequencyMod::simple(carrier, modulator, 1.0);

        // With zero modulator, should match original carrier
        assert!((fm.sample(0.25) - carrier.sample(0.25)).abs() < 0.001);
    }

    #[test]
    fn test_fm_modulates() {
        let carrier = Sine::with_frequency(1.0);
        let modulator = Sine::with_frequency(0.5);
        let fm = FrequencyMod::simple(carrier, modulator, 1.0);

        // FM output should differ from plain carrier at most times
        let fm_val = fm.sample(0.3);
        let carrier_val = carrier.sample(0.3);
        // They may occasionally match, so we just verify it's a valid value
        assert!((0.0..=1.0).contains(&fm_val));
        // At this specific time they should differ
        assert!((fm_val - carrier_val).abs() > 0.01 || fm_val.abs() < 0.1);
    }

    #[test]
    fn test_fm_zero_carrier_freq_falls_back() {
        let carrier = Sine::with_frequency(1.0);
        let modulator = Sine::with_frequency(0.5);
        let fm = FrequencyMod::new(carrier, modulator, 1.0, 0.0);

        let t = 0.37;
        assert!((fm.sample(t) - carrier.sample(t)).abs() < 0.001);
    }

    #[test]
    fn test_fm_non_finite_depth_falls_back() {
        let carrier = Sine::with_frequency(1.0);
        let modulator = Sine::with_frequency(0.5);
        let fm = FrequencyMod::new(carrier, modulator, f32::NAN, 1.0);

        let t = 0.37;
        assert!((fm.sample(t) - carrier.sample(t)).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/composition/cls_frequency_mod.rs</FILE> - <DESC>Frequency modulation operator</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
