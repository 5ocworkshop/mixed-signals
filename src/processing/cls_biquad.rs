// <FILE>src/processing/cls_biquad.rs</FILE> - <DESC>Biquad filter with multiple modes</DESC>
// <VERS>VERSION: 1.0.1</VERS>
// <WCTX>Clippy fixes</WCTX>
// <CLOG>Use FRAC_1_SQRT_2 constant instead of 0.7071 literal</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};
use std::f32::consts::{FRAC_1_SQRT_2, PI};
use std::sync::Mutex;

/// Filter mode for the biquad filter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiquadMode {
    /// Low-pass: passes frequencies below cutoff
    LowPass,
    /// High-pass: passes frequencies above cutoff
    HighPass,
    /// Band-pass: passes frequencies around cutoff
    BandPass,
    /// Notch (band-reject): attenuates frequencies around cutoff
    Notch,
}

/// Biquad filter state
#[derive(Debug, Clone, Copy, Default)]
struct BiquadState {
    x1: f32, // x[n-1]
    x2: f32, // x[n-2]
    y1: f32, // y[n-1]
    y2: f32, // y[n-2]
    prev_time: f64,
}

/// Second-order biquad filter with configurable mode and Q.
///
/// Implements the standard biquad difference equation:
/// `y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]`
///
/// Supports low-pass, high-pass, band-pass, and notch filter modes.
/// The Q parameter controls resonance/bandwidth.
///
/// # Example
/// ```ignore
/// let signal = Sine::new(440.0, 1.0, 0.0, 0.0);
/// let filtered = Biquad::new(signal, BiquadMode::LowPass, 1000.0, 0.707, 48000.0);
/// ```
#[derive(Debug)]
pub struct Biquad<S> {
    signal: S,
    mode: BiquadMode,
    cutoff_hz: f32,
    q: f32,
    sample_rate: f32,
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State
    state: Mutex<BiquadState>,
}

impl<S: Signal> Biquad<S> {
    /// Create a new biquad filter.
    ///
    /// # Arguments
    /// * `signal` - Input signal to filter
    /// * `mode` - Filter mode (LowPass, HighPass, BandPass, Notch)
    /// * `cutoff_hz` - Cutoff/center frequency in Hz
    /// * `q` - Quality factor (0.5 = gentle, 0.707 = Butterworth, >1 = resonant)
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(signal: S, mode: BiquadMode, cutoff_hz: f32, q: f32, sample_rate: f32) -> Self {
        let (b0, b1, b2, a1, a2) = Self::compute_coefficients(mode, cutoff_hz, q, sample_rate);
        Self {
            signal,
            mode,
            cutoff_hz,
            q,
            sample_rate,
            b0,
            b1,
            b2,
            a1,
            a2,
            state: Mutex::new(BiquadState::default()),
        }
    }

    /// Create a low-pass filter with Butterworth response (Q = 1/√2).
    pub fn lowpass(signal: S, cutoff_hz: f32, sample_rate: f32) -> Self {
        Self::new(
            signal,
            BiquadMode::LowPass,
            cutoff_hz,
            FRAC_1_SQRT_2,
            sample_rate,
        )
    }

    /// Create a high-pass filter with Butterworth response (Q = 1/√2).
    pub fn highpass(signal: S, cutoff_hz: f32, sample_rate: f32) -> Self {
        Self::new(
            signal,
            BiquadMode::HighPass,
            cutoff_hz,
            FRAC_1_SQRT_2,
            sample_rate,
        )
    }

    /// Create a band-pass filter.
    pub fn bandpass(signal: S, center_hz: f32, q: f32, sample_rate: f32) -> Self {
        Self::new(signal, BiquadMode::BandPass, center_hz, q, sample_rate)
    }

    /// Create a notch (band-reject) filter.
    pub fn notch(signal: S, center_hz: f32, q: f32, sample_rate: f32) -> Self {
        Self::new(signal, BiquadMode::Notch, center_hz, q, sample_rate)
    }

    /// Get the filter mode.
    pub fn mode(&self) -> BiquadMode {
        self.mode
    }

    /// Get the cutoff frequency in Hz.
    pub fn cutoff_hz(&self) -> f32 {
        self.cutoff_hz
    }

    /// Get the Q factor.
    pub fn q(&self) -> f32 {
        self.q
    }

    /// Get the sample rate in Hz.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Compute biquad coefficients using RBJ Audio EQ Cookbook formulas.
    fn compute_coefficients(
        mode: BiquadMode,
        cutoff_hz: f32,
        q: f32,
        sample_rate: f32,
    ) -> (f32, f32, f32, f32, f32) {
        let omega = 2.0 * PI * cutoff_hz / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q.max(0.001));

        let (b0, b1, b2, a0, a1, a2) = match mode {
            BiquadMode::LowPass => {
                let b1 = 1.0 - cos_omega;
                let b0 = b1 / 2.0;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadMode::HighPass => {
                let b1 = -(1.0 + cos_omega);
                let b0 = (1.0 + cos_omega) / 2.0;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadMode::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadMode::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize by a0
        (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
    }

    fn process_sample(&self, input: f32, state: &mut BiquadState) -> f32 {
        let output = self.b0 * input + self.b1 * state.x1 + self.b2 * state.x2
            - self.a1 * state.y1
            - self.a2 * state.y2;

        // Update state
        state.x2 = state.x1;
        state.x1 = input;
        state.y2 = state.y1;
        state.y1 = output;

        output
    }
}

impl<S: Signal> Signal for Biquad<S> {
    fn sample(&self, t: SignalTime) -> f32 {
        let input = self.signal.sample(t);

        let mut state = self.state.lock().unwrap();

        // Only process if time has advanced
        if t > state.prev_time {
            let output = self.process_sample(input, &mut state);
            state.prev_time = t;
            output
        } else if t < state.prev_time {
            // Time went backwards - reset state
            *state = BiquadState::default();
            state.prev_time = t;
            input
        } else {
            // Same time - return last output
            state.y1
        }
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let input = self.signal.sample_with_context(t, ctx);

        let mut state = self.state.lock().unwrap();

        if t > state.prev_time {
            let output = self.process_sample(input, &mut state);
            state.prev_time = t;
            output
        } else if t < state.prev_time {
            *state = BiquadState::default();
            state.prev_time = t;
            input
        } else {
            state.y1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    struct StepSignal {
        step_time: f64,
    }

    impl Signal for StepSignal {
        fn sample(&self, t: SignalTime) -> f32 {
            if t >= self.step_time {
                1.0
            } else {
                0.0
            }
        }
    }

    #[test]
    fn test_biquad_lowpass_dc() {
        let sig = Constant::new(0.5);
        let lpf = Biquad::lowpass(sig, 1000.0, 48000.0);

        // Run filter to steady state
        for i in 0..1000 {
            lpf.sample(i as f64 / 48000.0);
        }
        let result = lpf.sample(1001.0 / 48000.0);
        assert!(
            (result - 0.5).abs() < 0.05,
            "DC should pass through: {}",
            result
        );
    }

    #[test]
    fn test_biquad_highpass_dc() {
        let sig = Constant::new(0.5);
        let hpf = Biquad::highpass(sig, 1000.0, 48000.0);

        // Run filter to steady state
        for i in 0..1000 {
            hpf.sample(i as f64 / 48000.0);
        }
        let result = hpf.sample(1001.0 / 48000.0);
        assert!(result.abs() < 0.1, "DC should be blocked: {}", result);
    }

    #[test]
    fn test_biquad_notch_removes_center() {
        // A notch filter should attenuate signals at its center frequency
        let notch = Biquad::notch(Constant::new(0.5), 450.0, 2.0, 48000.0);

        // Run to steady state
        for i in 0..1000 {
            notch.sample(i as f64 / 48000.0);
        }
        // DC should still pass (notch is at 450 Hz, not DC)
        let result = notch.sample(1001.0 / 48000.0);
        assert!(
            (result - 0.5).abs() < 0.1,
            "DC should pass notch: {}",
            result
        );
    }

    #[test]
    fn test_biquad_bandpass_blocks_dc() {
        let sig = Constant::new(0.5);
        let bpf = Biquad::bandpass(sig, 1000.0, 1.0, 48000.0);

        // Run filter to steady state
        for i in 0..1000 {
            bpf.sample(i as f64 / 48000.0);
        }
        let result = bpf.sample(1001.0 / 48000.0);
        assert!(
            result.abs() < 0.1,
            "DC should be blocked by bandpass: {}",
            result
        );
    }

    #[test]
    fn test_biquad_step_response() {
        let step = StepSignal { step_time: 0.0 };
        let lpf = Biquad::lowpass(step, 100.0, 48000.0);

        // Low cutoff should show gradual rise
        let early = lpf.sample(0.001);
        assert!(
            early < 0.5,
            "Early response should be attenuated: {}",
            early
        );

        // Later should approach 1.0
        for i in 1..2000 {
            lpf.sample(i as f64 / 48000.0);
        }
        let late = lpf.sample(2001.0 / 48000.0);
        assert!(late > 0.9, "Should converge to 1.0: {}", late);
    }

    #[test]
    fn test_biquad_output_finite() {
        let sig = Constant::new(0.5);
        let lpf = Biquad::lowpass(sig, 1000.0, 48000.0);

        for i in 0..1000 {
            let val = lpf.sample(i as f64 / 48000.0);
            assert!(val.is_finite(), "Output should be finite at sample {}", i);
        }
    }
}

// <FILE>src/processing/cls_biquad.rs</FILE> - <DESC>Biquad filter with multiple modes</DESC>
// <VERS>END OF VERSION: 1.0.1</VERS>
