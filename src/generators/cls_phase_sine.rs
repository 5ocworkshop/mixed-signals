// <FILE>src/generators/cls_phase_sine.rs</FILE> - <DESC>Convert phase signal to sine wave</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Audio synthesis primitives - true FM support</WCTX>
// <CLOG>Initial implementation for phase-to-sine conversion</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};
use std::f32::consts::TAU;

/// Converts a phase signal [0, 1) to a sine wave [-1, 1].
///
/// This is the final stage of true FM synthesis:
/// 1. Create frequency signal: `center + deviation * modulator`
/// 2. Integrate with `PhaseAccumulator` → phase [0, 1)
/// 3. Convert with `PhaseSine` → sin(2π × phase) → [-1, 1]
///
/// # Examples
///
/// ```
/// use mixed_signals::generators::{PhaseSine, Constant};
/// use mixed_signals::traits::Signal;
///
/// // Phase 0 -> sin(0) = 0
/// // Phase 0.25 -> sin(π/2) = 1
/// // Phase 0.5 -> sin(π) = 0
/// // Phase 0.75 -> sin(3π/2) = -1
/// let phase = Constant::new(0.25);
/// let sine = PhaseSine::new(phase);
/// assert!((sine.sample(0.0) - 1.0).abs() < 0.001);
/// ```
#[derive(Debug, Clone)]
pub struct PhaseSine<P> {
    /// The phase signal [0, 1)
    pub phase: P,
}

impl<P: Signal> PhaseSine<P> {
    /// Create a new phase-to-sine converter.
    ///
    /// # Arguments
    /// * `phase` - Signal producing phase values [0, 1)
    pub fn new(phase: P) -> Self {
        Self { phase }
    }
}

impl<P: Signal> Signal for PhaseSine<P> {
    fn sample(&self, t: SignalTime) -> f32 {
        let phase = self.phase.sample(t);
        (TAU * phase).sin()
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let phase = self.phase.sample_with_context(t, ctx);
        (TAU * phase).sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    #[test]
    fn test_phase_sine_zero() {
        let phase = Constant::new(0.0);
        let sine = PhaseSine::new(phase);
        assert!((sine.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_sine_quarter() {
        let phase = Constant::new(0.25);
        let sine = PhaseSine::new(phase);
        assert!((sine.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_sine_half() {
        let phase = Constant::new(0.5);
        let sine = PhaseSine::new(phase);
        assert!((sine.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_sine_three_quarter() {
        let phase = Constant::new(0.75);
        let sine = PhaseSine::new(phase);
        assert!((sine.sample(0.0) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_phase_sine_full_cycle() {
        let phase = Constant::new(1.0);
        let sine = PhaseSine::new(phase);
        // sin(2π) = 0
        assert!((sine.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_sine_with_context() {
        let phase = Constant::new(0.25);
        let sine = PhaseSine::new(phase);
        let ctx = SignalContext::default();
        assert!((sine.sample_with_context(0.0, &ctx) - 1.0).abs() < 0.001);
    }
}

// <FILE>src/generators/cls_phase_sine.rs</FILE> - <DESC>Convert phase signal to sine wave</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
