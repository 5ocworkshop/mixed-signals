// <FILE>mixed-signals/src/processing/cls_lowpass.rs</FILE> - <DESC>One-pole low-pass filter</DESC>
// <VERS>VERSION: 1.0.0 - 2025-12-28</VERS>
// <WCTX>Larson Scanner audio implementation</WCTX>
// <CLOG>Initial implementation - one-pole IIR filter with cutoff frequency</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};
use std::f32::consts::PI;
use std::sync::Mutex;

/// One-pole low-pass filter for smoothing signals.
///
/// Implements: `y[n] = alpha * x[n] + (1 - alpha) * y[n-1]`
/// where alpha = 1 - exp(-2 * PI * cutoff / sample_rate)
///
/// This filter removes high-frequency content, creating a smoother,
/// "rounder" sound. Useful for softening harsh digital waveforms.
///
/// # Example
/// ```ignore
/// let triangle = Triangle::new(440.0, 1.0, 0.0, 0.0);
/// let smoothed = LowPass::new(triangle, 1000.0, 48000.0);
/// ```
#[derive(Debug)]
pub struct LowPass<S> {
    signal: S,
    /// Filter coefficient (0-1, higher = less filtering)
    alpha: f32,
    /// Filter state: (prev_output, prev_time)
    state: Mutex<(f32, SignalTime)>,
}

impl<S: Signal> LowPass<S> {
    /// Create a new low-pass filter.
    ///
    /// # Arguments
    /// * `signal` - Input signal to filter
    /// * `cutoff_hz` - Cutoff frequency in Hz (frequencies above this are attenuated)
    /// * `sample_rate` - Sample rate in Hz (e.g., 48000.0)
    pub fn new(signal: S, cutoff_hz: f32, sample_rate: f32) -> Self {
        // Compute filter coefficient
        // alpha = 1 - exp(-2 * PI * fc / fs)
        let alpha = 1.0 - (-2.0 * PI * cutoff_hz / sample_rate).exp();
        Self {
            signal,
            alpha: alpha.clamp(0.0, 1.0),
            state: Mutex::new((0.0, -1.0)), // (prev_output, prev_time)
        }
    }

    /// Create with a specific alpha coefficient (0-1).
    /// Lower alpha = more smoothing, higher alpha = less smoothing.
    pub fn with_alpha(signal: S, alpha: f32) -> Self {
        Self {
            signal,
            alpha: alpha.clamp(0.0, 1.0),
            state: Mutex::new((0.0, -1.0)),
        }
    }

    /// Get the current alpha coefficient
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
}

impl<S: Signal> Signal for LowPass<S> {
    fn sample(&self, t: SignalTime) -> f32 {
        let input = self.signal.sample(t);

        let mut state = self.state.lock().unwrap();
        let (prev_output, prev_time) = &mut *state;

        // Only update filter state if time has advanced
        if t > *prev_time {
            *prev_output = self.alpha * input + (1.0 - self.alpha) * *prev_output;
            *prev_time = t;
        } else if t < *prev_time {
            // Time went backwards (reset/seek) - reinitialize
            *prev_output = input;
            *prev_time = t;
        }
        // If t == prev_time, return cached value

        *prev_output
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let input = self.signal.sample_with_context(t, ctx);

        let mut state = self.state.lock().unwrap();
        let (prev_output, prev_time) = &mut *state;

        if t > *prev_time {
            *prev_output = self.alpha * input + (1.0 - self.alpha) * *prev_output;
            *prev_time = t;
        } else if t < *prev_time {
            *prev_output = input;
            *prev_time = t;
        }

        *prev_output
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
    fn test_lowpass_constant() {
        let sig = Constant::new(0.5);
        let lpf = LowPass::new(sig, 1000.0, 48000.0);

        // After several samples, should converge to input
        for i in 0..1000 {
            lpf.sample(i as f64 / 48000.0);
        }
        let result = lpf.sample(1000.0 / 48000.0);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lowpass_step_response() {
        let step = StepSignal { step_time: 0.0 };
        let lpf = LowPass::new(step, 100.0, 48000.0);

        // First sample should be attenuated
        let first = lpf.sample(0.0);
        assert!(first < 0.5, "First sample should be attenuated: {}", first);

        // Later samples should approach 1.0
        for i in 1..1000 {
            lpf.sample(i as f64 / 48000.0);
        }
        let later = lpf.sample(1000.0 / 48000.0);
        assert!(later > 0.9, "Should converge to 1.0: {}", later);
    }

    #[test]
    fn test_lowpass_high_cutoff_passes_signal() {
        let step = StepSignal { step_time: 0.0 };
        // Very high cutoff relative to sample rate = minimal filtering
        let lpf = LowPass::new(step, 20000.0, 48000.0);

        let result = lpf.sample(0.0);
        // High cutoff should pass most of the signal immediately
        assert!(result > 0.5, "High cutoff should pass signal: {}", result);
    }

    #[test]
    fn test_lowpass_low_cutoff_smooths_signal() {
        let step = StepSignal { step_time: 0.0 };
        // Very low cutoff = heavy smoothing
        let lpf = LowPass::new(step, 10.0, 48000.0);

        let result = lpf.sample(0.0);
        // Low cutoff should heavily attenuate
        assert!(result < 0.1, "Low cutoff should smooth heavily: {}", result);
    }
}

// <FILE>mixed-signals/src/processing/cls_lowpass.rs</FILE> - <DESC>One-pole low-pass filter</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-28</VERS>
