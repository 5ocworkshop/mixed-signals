// <FILE>src/traits/fnc_signal.rs</FILE> - <DESC>Closure wrapper for Signal trait</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Ergonomics improvements</WCTX>
// <CLOG>Initial creation - Fn1 wrapper for closures as signals</CLOG>

use super::{Signal, SignalContext, SignalTime};

/// Wrapper that allows a closure to be used as a Signal.
///
/// Since Rust doesn't allow blanket impls for `Fn(f32) -> f32`, this newtype
/// wrapper provides Signal functionality for arbitrary functions.
///
/// # Example
///
/// ```rust
/// use mixed_signals::traits::Fn1;
/// use mixed_signals::traits::Signal;
///
/// let custom = Fn1(|t: f64| (t * 3.14).sin() as f32 * 0.5 + 0.5);
/// let value = custom.sample(0.25);
/// assert!(value >= 0.0 && value <= 1.0);
/// ```
#[derive(Clone, Copy)]
pub struct Fn1<F>(pub F);

impl<F> std::fmt::Debug for Fn1<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fn1").finish_non_exhaustive()
    }
}

impl<F: Fn(SignalTime) -> f32 + Send + Sync> Signal for Fn1<F> {
    fn sample(&self, t: SignalTime) -> f32 {
        (self.0)(t).clamp(0.0, 1.0)
    }
}

/// Wrapper for context-aware closures.
///
/// For signals that need access to SignalContext.
///
/// # Example
///
/// ```rust
/// use mixed_signals::traits::{Fn2, Signal, SignalContext};
///
/// let custom = Fn2(|t: f64, ctx: &SignalContext| {
///     (t + ctx.frame as f64 * 0.01).fract() as f32
/// });
/// let ctx = SignalContext::new(100, 0);
/// let value = custom.sample_with_context(0.0, &ctx);
/// ```
#[derive(Clone, Copy)]
pub struct Fn2<F>(pub F);

impl<F> std::fmt::Debug for Fn2<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fn2").finish_non_exhaustive()
    }
}

impl<F: Fn(SignalTime, &SignalContext) -> f32 + Send + Sync> Signal for Fn2<F> {
    fn sample(&self, t: SignalTime) -> f32 {
        // For sample() without context, provide a default context
        (self.0)(t, &SignalContext::default()).clamp(0.0, 1.0)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        (self.0)(t, ctx).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn1_basic() {
        let sig = Fn1(|t: SignalTime| (t * 0.5) as f32);
        assert!((sig.sample(0.0) - 0.0).abs() < 0.001);
        assert!((sig.sample(1.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_fn1_clamps() {
        let sig = Fn1(|_t: SignalTime| 2.0); // Returns out of range
        assert_eq!(sig.sample(0.0), 1.0); // Clamped to 1.0
    }

    #[test]
    fn test_fn1_sine() {
        let sig = Fn1(|t: SignalTime| (t * std::f64::consts::TAU).sin() as f32 * 0.5 + 0.5);
        assert!((sig.sample(0.0) - 0.5).abs() < 0.001);
        assert!((sig.sample(0.25) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fn2_with_context() {
        let sig =
            Fn2(|t: SignalTime, ctx: &SignalContext| (t + ctx.frame as SignalTime * 0.1) as f32);
        let ctx = SignalContext::new(5, 0);
        assert!((sig.sample_with_context(0.0, &ctx) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_fn1_debug() {
        let sig = Fn1(|t: SignalTime| t as f32);
        let debug = format!("{:?}", sig);
        assert!(debug.contains("Fn1"));
    }
}

// <FILE>src/traits/fnc_signal.rs</FILE> - <DESC>Closure wrapper for Signal trait</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
