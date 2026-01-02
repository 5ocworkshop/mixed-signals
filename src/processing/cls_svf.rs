// <FILE>src/processing/cls_svf.rs</FILE> - <DESC>State Variable Filter with dynamic cutoff</DESC>
// <VERS>VERSION: 1.0.1</VERS>
// <WCTX>Clippy fixes</WCTX>
// <CLOG>Use += operators instead of manual assign operations</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};
use std::f32::consts::PI;
use std::sync::Mutex;

/// Output mode for the SVF.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvfMode {
    /// Low-pass output
    LowPass,
    /// High-pass output
    HighPass,
    /// Band-pass output
    BandPass,
}

/// SVF internal state
#[derive(Debug, Clone, Copy, Default)]
struct SvfState {
    low: f32,
    band: f32,
    prev_time: f64,
}

/// State Variable Filter with dynamic cutoff frequency.
///
/// The SVF is a classic analog-modeled filter that provides simultaneous
/// low-pass, high-pass, and band-pass outputs. This implementation uses
/// the Chamberlin topology.
///
/// Unlike `Biquad`, this filter accepts a signal for the cutoff frequency,
/// enabling filter sweeps and modulation.
///
/// # Example
/// ```ignore
/// let noise = WhiteNoise::with_seed(42);
/// let cutoff = Keyframes::from_pairs(&[(0.0, 400.0), (1.0, 800.0)]);
/// let filtered = Svf::bandpass(noise, cutoff, 15.0, 48000.0);
/// ```
#[derive(Debug)]
pub struct Svf<S, C> {
    signal: S,
    cutoff: C,
    q: f32,
    sample_rate: f32,
    mode: SvfMode,
    state: Mutex<SvfState>,
}

impl<S: Signal, C: Signal> Svf<S, C> {
    /// Create a new SVF.
    ///
    /// # Arguments
    /// * `signal` - Input signal to filter
    /// * `cutoff` - Cutoff frequency signal in Hz
    /// * `q` - Quality factor (resonance). Higher = narrower bandwidth
    /// * `sample_rate` - Sample rate in Hz
    /// * `mode` - Output mode (LowPass, HighPass, BandPass)
    pub fn new(signal: S, cutoff: C, q: f32, sample_rate: f32, mode: SvfMode) -> Self {
        Self {
            signal,
            cutoff,
            q: q.max(0.5),
            sample_rate,
            mode,
            state: Mutex::new(SvfState::default()),
        }
    }

    /// Create a low-pass SVF.
    pub fn lowpass(signal: S, cutoff: C, q: f32, sample_rate: f32) -> Self {
        Self::new(signal, cutoff, q, sample_rate, SvfMode::LowPass)
    }

    /// Create a high-pass SVF.
    pub fn highpass(signal: S, cutoff: C, q: f32, sample_rate: f32) -> Self {
        Self::new(signal, cutoff, q, sample_rate, SvfMode::HighPass)
    }

    /// Create a band-pass SVF.
    pub fn bandpass(signal: S, cutoff: C, q: f32, sample_rate: f32) -> Self {
        Self::new(signal, cutoff, q, sample_rate, SvfMode::BandPass)
    }

    fn process_sample(&self, input: f32, cutoff_hz: f32, state: &mut SvfState) -> f32 {
        // Chamberlin SVF
        // f = 2 * sin(pi * fc / fs)
        // For stability, clamp cutoff to Nyquist
        let fc = cutoff_hz.clamp(20.0, self.sample_rate * 0.49);
        let f = 2.0 * (PI * fc / self.sample_rate).sin();
        let q_inv = 1.0 / self.q;

        // Update state
        state.low += f * state.band;
        let high = input - state.low - q_inv * state.band;
        state.band += f * high;

        // Return selected output
        match self.mode {
            SvfMode::LowPass => state.low,
            SvfMode::HighPass => high,
            SvfMode::BandPass => state.band,
        }
    }
}

impl<S: Signal, C: Signal> Signal for Svf<S, C> {
    fn sample(&self, t: SignalTime) -> f32 {
        let input = self.signal.sample(t);
        let cutoff_hz = self.cutoff.sample(t);

        let mut state = self.state.lock().unwrap();

        if t > state.prev_time {
            let output = self.process_sample(input, cutoff_hz, &mut state);
            state.prev_time = t;
            output
        } else if t < state.prev_time {
            // Time went backwards - reset
            *state = SvfState::default();
            state.prev_time = t;
            input
        } else {
            // Same time
            match self.mode {
                SvfMode::LowPass => state.low,
                SvfMode::BandPass => state.band,
                SvfMode::HighPass => input - state.low - state.band / self.q,
            }
        }
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let input = self.signal.sample_with_context(t, ctx);
        let cutoff_hz = self.cutoff.sample_with_context(t, ctx);

        let mut state = self.state.lock().unwrap();

        if t > state.prev_time {
            let output = self.process_sample(input, cutoff_hz, &mut state);
            state.prev_time = t;
            output
        } else if t < state.prev_time {
            *state = SvfState::default();
            state.prev_time = t;
            input
        } else {
            match self.mode {
                SvfMode::LowPass => state.low,
                SvfMode::BandPass => state.band,
                SvfMode::HighPass => input - state.low - state.band / self.q,
            }
        }
    }
}

/// Convenience type for SVF with constant cutoff frequency.
pub type SvfFixed<S> = Svf<S, crate::generators::Constant>;

impl<S: Signal> SvfFixed<S> {
    /// Create a band-pass SVF with fixed cutoff.
    pub fn bandpass_fixed(signal: S, cutoff_hz: f32, q: f32, sample_rate: f32) -> Self {
        use crate::generators::Constant;
        Svf::bandpass(signal, Constant::new(cutoff_hz), q, sample_rate)
    }

    /// Create a low-pass SVF with fixed cutoff.
    pub fn lowpass_fixed(signal: S, cutoff_hz: f32, q: f32, sample_rate: f32) -> Self {
        use crate::generators::Constant;
        Svf::lowpass(signal, Constant::new(cutoff_hz), q, sample_rate)
    }

    /// Create a high-pass SVF with fixed cutoff.
    pub fn highpass_fixed(signal: S, cutoff_hz: f32, q: f32, sample_rate: f32) -> Self {
        use crate::generators::Constant;
        Svf::highpass(signal, Constant::new(cutoff_hz), q, sample_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    #[test]
    fn test_svf_lowpass_dc() {
        let sig = Constant::new(0.5);
        let cutoff = Constant::new(1000.0);
        let lpf = Svf::lowpass(sig, cutoff, 0.707, 48000.0);

        // Run to steady state
        for i in 0..2000 {
            lpf.sample(i as f64 / 48000.0);
        }
        let result = lpf.sample(2001.0 / 48000.0);
        assert!((result - 0.5).abs() < 0.1, "DC should pass LP: {}", result);
    }

    #[test]
    fn test_svf_highpass_dc() {
        let sig = Constant::new(0.5);
        let cutoff = Constant::new(1000.0);
        let hpf = Svf::highpass(sig, cutoff, 0.707, 48000.0);

        // Run to steady state
        for i in 0..2000 {
            hpf.sample(i as f64 / 48000.0);
        }
        let result = hpf.sample(2001.0 / 48000.0);
        assert!(result.abs() < 0.1, "DC should be blocked by HP: {}", result);
    }

    #[test]
    fn test_svf_bandpass_dc() {
        let sig = Constant::new(0.5);
        let cutoff = Constant::new(1000.0);
        let bpf = Svf::bandpass(sig, cutoff, 1.0, 48000.0);

        // Run to steady state
        for i in 0..2000 {
            bpf.sample(i as f64 / 48000.0);
        }
        let result = bpf.sample(2001.0 / 48000.0);
        assert!(result.abs() < 0.1, "DC should be blocked by BP: {}", result);
    }

    #[test]
    fn test_svf_high_q_resonance() {
        let sig = Constant::new(0.5);
        let cutoff = Constant::new(1000.0);
        let bpf_low_q = Svf::bandpass(sig, cutoff, 1.0, 48000.0);
        let bpf_high_q = Svf::bandpass(sig, cutoff, 15.0, 48000.0);

        // Higher Q should show different transient behavior
        let low_q_val = bpf_low_q.sample(0.001);
        let high_q_val = bpf_high_q.sample(0.001);
        // Both should be finite
        assert!(low_q_val.is_finite());
        assert!(high_q_val.is_finite());
    }

    #[test]
    fn test_svf_output_finite() {
        let sig = Constant::new(0.5);
        let cutoff = Constant::new(1000.0);
        let bpf = Svf::bandpass(sig, cutoff, 15.0, 48000.0);

        for i in 0..1000 {
            let val = bpf.sample(i as f64 / 48000.0);
            assert!(val.is_finite(), "Output should be finite at sample {}", i);
        }
    }

    #[test]
    fn test_svf_fixed_convenience() {
        let sig = Constant::new(0.5);
        let bpf = SvfFixed::bandpass_fixed(sig, 1000.0, 15.0, 48000.0);

        let val = bpf.sample(0.01);
        assert!(val.is_finite());
    }
}

// <FILE>src/processing/cls_svf.rs</FILE> - <DESC>State Variable Filter with dynamic cutoff</DESC>
// <VERS>END OF VERSION: 1.0.1</VERS>
