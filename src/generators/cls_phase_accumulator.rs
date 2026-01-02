// <FILE>mixed-signals/src/generators/cls_phase_accumulator.rs</FILE> - <DESC>Phase accumulator for FM synthesis</DESC>
// <VERS>VERSION: 1.2.0</VERS>
// <WCTX>Audio synthesis primitives - true frequency modulation</WCTX>
// <CLOG>Fixed trapezoidal integration: prev_freq was shadowed and never updated</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};

/// Phase accumulator for true frequency modulation synthesis.
///
/// Integrates a frequency signal over time to produce a phase signal,
/// enabling true FM synthesis. This differs from the library's `FrequencyMod`
/// which implements phase modulation.
///
/// # Output
///
/// Returns accumulated phase normalized to [0.0, 1.0), where 1.0 represents
/// a full cycle. The phase wraps at 1.0.
///
/// # Stateless Implementation
///
/// The Signal trait requires pure functions. For constant frequency, exact
/// integration is trivial: `phase = initial_phase + freq * t`. For varying
/// frequency, we approximate using the current frequency value.
///
/// For slowly varying frequency signals (LFOs, envelopes), this approximation
/// is accurate. For rapidly varying signals, precision decreases.
///
/// # Examples
///
/// ```
/// use mixed_signals::generators::{PhaseAccumulator, Constant};
/// use mixed_signals::traits::Signal;
///
/// // 1 Hz frequency -> phase cycles 0 to 1 over 1 second
/// let freq = Constant::new(1.0);
/// let phase = PhaseAccumulator::new(freq, 0.0);
///
/// assert!((phase.sample(0.0) - 0.0).abs() < 0.001);
/// assert!((phase.sample(0.25) - 0.25).abs() < 0.001);
/// assert!((phase.sample(0.5) - 0.5).abs() < 0.001);
/// ```
#[derive(Debug, Clone)]
pub struct PhaseAccumulator<F> {
    /// The frequency signal (in Hz)
    pub frequency: F,
    /// Initial phase offset [0.0, 1.0)
    pub initial_phase: f32,
}

impl<F: Signal> PhaseAccumulator<F> {
    /// Create a new phase accumulator.
    ///
    /// # Arguments
    /// * `frequency` - Signal producing frequency values in Hz
    /// * `initial_phase` - Starting phase [0.0, 1.0)
    pub fn new(frequency: F, initial_phase: f32) -> Self {
        Self {
            frequency,
            initial_phase: initial_phase.rem_euclid(1.0),
        }
    }

    /// Create with zero initial phase.
    pub fn with_frequency(frequency: F) -> Self {
        Self::new(frequency, 0.0)
    }
}

impl<F: Signal> Signal for PhaseAccumulator<F> {
    fn sample(&self, t: SignalTime) -> f32 {
        if !t.is_finite() || t < 0.0 {
            return self.initial_phase;
        }

        // Numerical integration of frequency over time using trapezoidal rule.
        // For FM synthesis, we need: phase = ∫freq(τ)dτ from 0 to t
        //
        // Use ~1000 steps per second for audio-quality integration.
        // This balances accuracy with performance.
        let steps_per_second = 1000.0;
        let num_steps = ((t * steps_per_second).ceil() as usize).max(1);
        let dt = t / num_steps as f64;

        let mut accumulated_phase = self.initial_phase as f64;
        let freq_0 = self.frequency.sample(0.0);
        let mut prev_freq = if freq_0.is_finite() { freq_0 } else { 0.0 };

        for i in 1..=num_steps {
            let t_i = i as f64 * dt;
            let freq_i = self.frequency.sample(t_i);
            let curr_freq = if freq_i.is_finite() { freq_i } else { 0.0 };

            // Trapezoidal rule: (f(a) + f(b)) / 2 * dt
            accumulated_phase += (prev_freq + curr_freq) as f64 / 2.0 * dt;
            prev_freq = curr_freq;
        }

        // Wrap to [0, 1) - handle floating point edge case at exactly 0
        let phase = accumulated_phase as f32;
        let wrapped = phase.rem_euclid(1.0);
        // If very close to 1.0 due to floating point, treat as 0.0
        if (wrapped - 1.0).abs() < 1e-6 {
            0.0
        } else {
            wrapped
        }
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        if !t.is_finite() || t < 0.0 {
            return self.initial_phase;
        }

        let steps_per_second = 1000.0;
        let num_steps = ((t * steps_per_second).ceil() as usize).max(1);
        let dt = t / num_steps as f64;

        let mut accumulated_phase = self.initial_phase as f64;
        let freq_0 = self.frequency.sample_with_context(0.0, ctx);
        let mut prev_freq = if freq_0.is_finite() { freq_0 } else { 0.0 };

        for i in 1..=num_steps {
            let t_i = i as f64 * dt;
            let freq_i = self.frequency.sample_with_context(t_i, ctx);
            let curr_freq = if freq_i.is_finite() { freq_i } else { 0.0 };

            accumulated_phase += (prev_freq + curr_freq) as f64 / 2.0 * dt;
            prev_freq = curr_freq;
        }

        // Wrap to [0, 1) - handle floating point edge case at exactly 0
        let phase = accumulated_phase as f32;
        let wrapped = phase.rem_euclid(1.0);
        if (wrapped - 1.0).abs() < 1e-6 {
            0.0
        } else {
            wrapped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    /// Test helper: an unclamped frequency signal that returns raw Hz values
    struct RawFrequency(f32);
    impl Signal for RawFrequency {
        fn sample(&self, _t: SignalTime) -> f32 {
            self.0
        }
    }

    #[test]
    fn test_phase_accumulator_constant_frequency() {
        let freq = Constant::new(1.0); // 1 Hz (within [0,1] so no clamping)
        let phase = PhaseAccumulator::new(freq, 0.0);

        // At t=0, phase should be 0
        assert!((phase.sample(0.0) - 0.0).abs() < 0.001);

        // At t=0.25, phase should be 0.25
        assert!((phase.sample(0.25) - 0.25).abs() < 0.001);

        // At t=0.5, phase should be 0.5
        assert!((phase.sample(0.5) - 0.5).abs() < 0.001);

        // At t=1.0, phase should wrap to 0
        assert!((phase.sample(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_2hz() {
        let freq = RawFrequency(2.0); // 2 Hz (unclamped)
        let phase = PhaseAccumulator::new(freq, 0.0);

        // At t=0.25, phase should be 0.5 (half cycle at 2Hz)
        assert!((phase.sample(0.25) - 0.5).abs() < 0.001);

        // At t=0.5, phase should wrap to 0
        assert!((phase.sample(0.5) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_with_initial_phase() {
        let freq = Constant::new(1.0);
        let phase = PhaseAccumulator::new(freq, 0.25);

        // At t=0, phase should be initial_phase
        assert!((phase.sample(0.0) - 0.25).abs() < 0.001);

        // At t=0.5, phase should be 0.75
        assert!((phase.sample(0.5) - 0.75).abs() < 0.001);

        // At t=0.75, phase should wrap: 0.25 + 0.75 = 1.0 -> 0.0
        assert!((phase.sample(0.75) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_zero_frequency() {
        let freq = Constant::new(0.0);
        let phase = PhaseAccumulator::new(freq, 0.3);

        // Phase should remain at initial value
        assert!((phase.sample(0.0) - 0.3).abs() < 0.001);
        assert!((phase.sample(1.0) - 0.3).abs() < 0.001);
        assert!((phase.sample(10.0) - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_negative_frequency() {
        let freq = RawFrequency(-1.0); // Negative 1 Hz (unclamped)
        let phase = PhaseAccumulator::new(freq, 0.5);

        // At t=0, phase should be 0.5
        assert!((phase.sample(0.0) - 0.5).abs() < 0.001);

        // At t=0.25, phase should be 0.25 (going backwards)
        let v_25 = phase.sample(0.25);
        assert!(
            (v_25 - 0.25).abs() < 0.001,
            "At t=0.25: expected 0.25, got {}",
            v_25
        );

        // At t=0.5, phase should be 0.0 (wrapped from negative)
        let v_50 = phase.sample(0.5);
        assert!(
            (v_50 - 0.0).abs() < 0.001,
            "At t=0.5: expected 0.0, got {}",
            v_50
        );
    }

    #[test]
    fn test_phase_accumulator_wrapping() {
        let freq = Constant::new(1.0);
        let phase = PhaseAccumulator::new(freq, 0.0);

        for i in 0..100 {
            let t = i as f64 * 0.1;
            let v = phase.sample(t);
            assert!(
                (0.0..1.0).contains(&v),
                "Phase {} should be in [0, 1) at t={}",
                v,
                t
            );
        }
    }

    #[test]
    fn test_phase_accumulator_nan_frequency() {
        struct NanSignal;
        impl Signal for NanSignal {
            fn sample(&self, _t: SignalTime) -> f32 {
                f32::NAN
            }
        }

        let phase = PhaseAccumulator::new(NanSignal, 0.25);
        let v = phase.sample(1.0);

        // NaN frequency treated as 0, so phase stays at initial
        assert!((v - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_negative_time() {
        let freq = Constant::new(1.0);
        let phase = PhaseAccumulator::new(freq, 0.3);

        // Negative time should return initial phase
        assert!((phase.sample(-1.0) - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_with_context() {
        let freq = Constant::new(1.0);
        let phase = PhaseAccumulator::new(freq, 0.0);
        let ctx = SignalContext::default();

        assert!((phase.sample_with_context(0.25, &ctx) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_phase_accumulator_initial_phase_normalization() {
        let freq = Constant::new(1.0);

        // Initial phase > 1 should be normalized
        let phase = PhaseAccumulator::new(freq, 1.5);
        assert!((phase.initial_phase - 0.5).abs() < 0.001);

        // Negative initial phase should be normalized
        let phase = PhaseAccumulator::new(freq, -0.25);
        assert!((phase.initial_phase - 0.75).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/generators/cls_phase_accumulator.rs</FILE> - <DESC>Phase accumulator for FM synthesis</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
