// <FILE>mixed-signals/src/traits/signal.rs</FILE> - <DESC>Core Signal trait definition</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>mixed-signals extraction: Universal Phase implementation (Option E)</WCTX>
// <CLOG>Replaced AnimationPhase with universal Phase enum for broader applicability</CLOG>

/// Universal phase model for lifecycle-aware signal evaluation.
///
/// This enum provides semantic phases that map to most lifecycle models:
/// - **Animations**: Start=entering, Active=dwelling, End=exiting, Done=finished
/// - **Audio ADSR**: Start=attack, Active=sustain, End=release, Done=silent
/// - **Games**: Start=loading, Active=playing, End=game_over, Custom(n)=domain-specific states
/// - **Physics**: Start=initialization, Active=simulation, End=cleanup
///
/// The `Custom` variant allows applications to define domain-specific phases
/// beyond the common lifecycle model (0-251 for custom states).
///
/// # Examples
///
/// ```rust
/// use mixed_signals::traits::Phase;
///
/// // Animation use case
/// let phase = Phase::Start;  // Entering/fade-in
///
/// // Game use case
/// let menu = Phase::Custom(1);
/// let cutscene = Phase::Custom(2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Beginning/initialization/fade-in phase
    Start,
    /// Main active/steady-state phase
    #[default]
    Active,
    /// Ending/cleanup/fade-out phase
    End,
    /// Completed/terminal state
    Done,
    /// Custom application-defined phase (0-251 for domain-specific states)
    Custom(u8),
}

/// Context for signal evaluation, providing additional information
/// for deterministic noise and spatial effects.
#[derive(Debug, Clone, Default)]
pub struct SignalContext {
    /// Frame number for deterministic randomness
    pub frame: u64,
    /// Seed for reproducible noise
    pub seed: u64,
    /// Width of the render area (for spatial signals)
    pub width: u16,
    /// Height of the render area (for spatial signals)
    pub height: u16,

    // WG8 additions: Phase-aware time context
    /// Current lifecycle phase (Start, Active, End, Done, or Custom)
    pub phase: Option<Phase>,
    /// Progress through current phase (0.0-1.0, resets at phase transitions)
    pub phase_t: Option<SignalTime>,
    /// Cyclical loop time (0.0-1.0, repeating based on loop_period)
    pub loop_t: Option<SignalTime>,
    /// Total elapsed time since animation start (seconds, monotonically increasing)
    pub absolute_t: Option<SignalTime>,

    // feat-20251224-155211: Per-character context
    /// Character index for per-character signal evaluation (used by PerCharacterNoise)
    pub char_index: Option<usize>,
}

impl SignalContext {
    pub fn new(frame: u64, seed: u64) -> Self {
        Self {
            frame,
            seed,
            width: 0,
            height: 0,
            phase: None,
            phase_t: None,
            loop_t: None,
            absolute_t: None,
            char_index: None,
        }
    }

    pub fn with_dimensions(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_char_index(mut self, char_index: usize) -> Self {
        self.char_index = Some(char_index);
        self
    }

    /// Context for phase-based effects (easing, entrance/exit animations)
    pub fn for_phase(phase: Phase, phase_t: SignalTime, frame: u64) -> Self {
        Self {
            frame,
            seed: 0,
            width: 0,
            height: 0,
            phase: Some(phase),
            phase_t: Some(phase_t.clamp(0.0, 1.0)),
            loop_t: None,
            absolute_t: None,
            char_index: None,
        }
    }

    /// Context for looping effects (pulsing, oscillating)
    pub fn for_loop(loop_t: SignalTime, frame: u64) -> Self {
        Self {
            frame,
            seed: 0,
            width: 0,
            height: 0,
            phase: None,
            phase_t: None,
            loop_t: Some(loop_t.clamp(0.0, 1.0)),
            absolute_t: None,
            char_index: None,
        }
    }

    /// Full context with both phase and loop time
    pub fn full(
        phase: Phase,
        phase_t: SignalTime,
        loop_t: Option<SignalTime>,
        absolute_t: SignalTime,
        frame: u64,
    ) -> Self {
        Self {
            frame,
            seed: 0,
            width: 0,
            height: 0,
            phase: Some(phase),
            phase_t: Some(phase_t.clamp(0.0, 1.0)),
            loop_t: loop_t.map(|value| value.clamp(0.0, 1.0)),
            absolute_t: Some(absolute_t.max(0.0)),
            char_index: None,
        }
    }
}

/// Canonical time parameter type used across mixed-signals.
///
/// Signal outputs remain f32, but time inputs are f64 to avoid precision loss
/// over long-running sessions.
pub type SignalTime = f64;

/// Expected output range for a signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalRange {
    pub min: f32,
    pub max: f32,
}

impl SignalRange {
    pub const UNIT: SignalRange = SignalRange { min: 0.0, max: 1.0 };
    pub const BIPOLAR: SignalRange = SignalRange {
        min: -1.0,
        max: 1.0,
    };

    pub fn new(min: f32, max: f32) -> Self {
        if !min.is_finite() || !max.is_finite() {
            return SignalRange::UNIT;
        }
        if min <= max {
            Self { min, max }
        } else {
            Self { min: max, max: min }
        }
    }
}

/// A signal produces a value given a time input.
///
/// Signals are the fundamental building blocks for driving animations.
/// They can represent oscillators, noise, envelopes, or composed signals.
///
/// # Output Range
///
/// By convention, signals output values in the unit range (0.0 to 1.0).
/// Use processing operators like `Remap` or `Clamp` if you need other ranges.
/// Override `output_range()` to describe a different expected range.
///
/// # Time Units
///
/// The `t` parameter can represent:
/// - Seconds (for frequency-based oscillators)
/// - Normalized progress (0.0 to 1.0 for animation-based signals)
///
/// The interpretation depends on the signal type and configuration.
pub trait Signal: Send + Sync {
    /// Report the expected output range for this signal.
    ///
    /// Default is the unit range (0.0..1.0).
    fn output_range(&self) -> SignalRange {
        SignalRange::UNIT
    }

    /// Sample the signal at time t.
    ///
    /// # Arguments
    /// * `t` - Time value (seconds or normalized progress)
    ///
    /// # Returns
    /// The signal value at time t
    fn sample(&self, t: SignalTime) -> f32;

    /// Sample the signal with additional context.
    ///
    /// Some signals (like noise generators) benefit from context
    /// for deterministic behavior.
    ///
    /// Default implementation ignores context and calls `sample`.
    fn sample_with_context(&self, t: SignalTime, _ctx: &SignalContext) -> f32 {
        self.sample(t)
    }

    /// Sample the signal into a pre-allocated buffer.
    ///
    /// Values are sampled starting at `t_start` and incrementing by `dt`.
    fn sample_into(&self, t_start: SignalTime, dt: SignalTime, out: &mut [f32]) {
        let mut t = t_start;
        for value in out.iter_mut() {
            *value = self.sample(t);
            t += dt;
        }
    }

    /// Sample the signal into a pre-allocated buffer with context.
    ///
    /// Values are sampled starting at `t_start` and incrementing by `dt`.
    fn sample_with_context_into(
        &self,
        t_start: SignalTime,
        dt: SignalTime,
        ctx: &SignalContext,
        out: &mut [f32],
    ) {
        let mut t = t_start;
        for value in out.iter_mut() {
            *value = self.sample_with_context(t, ctx);
            t += dt;
        }
    }

    /// Convenience helper that returns a Vec of sampled values.
    fn sample_vec(&self, t_start: SignalTime, dt: SignalTime, count: usize) -> Vec<f32> {
        let mut values = vec![0.0; count];
        self.sample_into(t_start, dt, &mut values);
        values
    }
}

// Allow boxed signals to be used as signals
impl Signal for Box<dyn Signal> {
    fn output_range(&self) -> SignalRange {
        (**self).output_range()
    }

    fn sample(&self, t: SignalTime) -> f32 {
        (**self).sample(t)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        (**self).sample_with_context(t, ctx)
    }
}

// Allow Arc<dyn Signal> to be used as signals
impl Signal for std::sync::Arc<dyn Signal> {
    fn output_range(&self) -> SignalRange {
        (**self).output_range()
    }

    fn sample(&self, t: SignalTime) -> f32 {
        (**self).sample(t)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        (**self).sample_with_context(t, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstantSignal(f32);

    impl Signal for ConstantSignal {
        fn sample(&self, _t: SignalTime) -> f32 {
            self.0
        }
    }

    #[test]
    fn test_basic_signal() {
        let sig = ConstantSignal(0.5);
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
        assert!((sig.sample(1.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_boxed_signal() {
        let sig: Box<dyn Signal> = Box::new(ConstantSignal(0.75));
        assert!((sig.sample(0.0) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_context_default() {
        let sig = ConstantSignal(0.5);
        let ctx = SignalContext::default();
        assert!((sig.sample_with_context(0.0, &ctx) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_context_phase_clamps() {
        let ctx = SignalContext::for_phase(Phase::Active, 1.5, 0);
        assert_eq!(ctx.phase_t, Some(1.0));

        let ctx = SignalContext::for_phase(Phase::Active, -0.5, 0);
        assert_eq!(ctx.phase_t, Some(0.0));
    }

    #[test]
    fn test_context_loop_clamps() {
        let ctx = SignalContext::for_loop(1.2, 0);
        assert_eq!(ctx.loop_t, Some(1.0));

        let ctx = SignalContext::for_loop(-0.2, 0);
        assert_eq!(ctx.loop_t, Some(0.0));
    }

    #[test]
    fn test_context_absolute_non_negative() {
        let ctx = SignalContext::full(Phase::Active, 0.5, None, -1.0, 0);
        assert_eq!(ctx.absolute_t, Some(0.0));
    }

    #[test]
    fn test_sample_into_uniform() {
        let sig = ConstantSignal(0.25);
        let mut out = [0.0; 4];
        sig.sample_into(0.0, 0.1, &mut out);
        for value in out.iter() {
            assert!((*value - 0.25).abs() < 1e-6);
        }
    }

    #[test]
    fn test_sample_with_context_into() {
        struct ContextSignal;

        impl Signal for ContextSignal {
            fn sample(&self, t: SignalTime) -> f32 {
                t as f32
            }

            fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
                (t + ctx.frame as SignalTime) as f32
            }
        }

        let sig = ContextSignal;
        let ctx = SignalContext::new(2, 0);
        let mut out = [0.0; 3];
        sig.sample_with_context_into(0.0, 0.5, &ctx, &mut out);
        assert!((out[0] - 2.0).abs() < 1e-6);
        assert!((out[1] - 2.5).abs() < 1e-6);
        assert!((out[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_signal_range_non_finite_defaults_unit() {
        let range = SignalRange::new(f32::NAN, 1.0);
        assert_eq!(range, SignalRange::UNIT);
    }
}

// <FILE>mixed-signals/src/traits/signal.rs</FILE> - <DESC>Core Signal trait definition</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-23</VERS>
