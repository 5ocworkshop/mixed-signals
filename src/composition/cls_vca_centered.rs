// <FILE>mixed-signals/src/composition/cls_vca_centered.rs</FILE> - <DESC>Centered voltage-controlled amplifier</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Audio synthesis primitives - VCA with centered silence</WCTX>
// <CLOG>Initial implementation - VCA that maintains 0.5 center when amplitude is zero</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};

/// Centered voltage-controlled amplifier (VCA).
///
/// Unlike simple multiplication, this VCA maintains a neutral center point (0.5)
/// when amplitude is zero, avoiding the "silence becomes -1" problem.
///
/// Formula: `output = carrier * amplitude + 0.5 * (1.0 - amplitude)`
///
/// - When amplitude = 0.0: output = 0.5 (neutral/silence)
/// - When amplitude = 1.0: output = carrier
/// - When amplitude = 0.5: output = lerp(0.5, carrier, 0.5)
///
/// # Examples
///
/// ```
/// use mixed_signals::composition::VcaCentered;
/// use mixed_signals::generators::Constant;
/// use mixed_signals::traits::Signal;
///
/// let carrier = Constant::new(1.0);
/// let amplitude = Constant::new(0.0);
/// let vca = VcaCentered::new(carrier, amplitude);
///
/// // With zero amplitude, output is centered at 0.5
/// assert_eq!(vca.sample(0.0), 0.5);
/// ```
#[derive(Debug, Clone)]
pub struct VcaCentered<C, A> {
    /// The carrier signal to be amplitude-modulated
    pub carrier: C,
    /// The amplitude control signal (0..1)
    pub amplitude: A,
}

impl<C: Signal, A: Signal> VcaCentered<C, A> {
    /// Create a new centered VCA.
    ///
    /// # Arguments
    /// * `carrier` - The signal to modulate
    /// * `amplitude` - The amplitude control signal (0..1 range)
    pub fn new(carrier: C, amplitude: A) -> Self {
        Self { carrier, amplitude }
    }
}

impl<C: Signal, A: Signal> Signal for VcaCentered<C, A> {
    fn sample(&self, t: SignalTime) -> f32 {
        let c = self.carrier.sample(t);
        let a = self.amplitude.sample(t);

        // Handle NaN: treat as neutral
        let c = if c.is_finite() {
            c.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let a = if a.is_finite() {
            a.clamp(0.0, 1.0)
        } else {
            0.0
        };

        // VCA formula: carrier * amp + center * (1 - amp)
        // This is equivalent to lerp(0.5, carrier, amplitude)
        c * a + 0.5 * (1.0 - a)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let c = self.carrier.sample_with_context(t, ctx);
        let a = self.amplitude.sample_with_context(t, ctx);

        let c = if c.is_finite() {
            c.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let a = if a.is_finite() {
            a.clamp(0.0, 1.0)
        } else {
            0.0
        };

        c * a + 0.5 * (1.0 - a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};

    #[test]
    fn test_vca_zero_amplitude_gives_center() {
        let carrier = Constant::new(1.0);
        let amplitude = Constant::zero();
        let vca = VcaCentered::new(carrier, amplitude);

        assert_eq!(vca.sample(0.0), 0.5);
        assert_eq!(vca.sample(0.5), 0.5);
        assert_eq!(vca.sample(1.0), 0.5);
    }

    #[test]
    fn test_vca_full_amplitude_gives_carrier() {
        let carrier = Constant::new(0.8);
        let amplitude = Constant::one();
        let vca = VcaCentered::new(carrier, amplitude);

        assert!((vca.sample(0.0) - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_vca_half_amplitude() {
        let carrier = Constant::new(1.0);
        let amplitude = Constant::new(0.5);
        let vca = VcaCentered::new(carrier, amplitude);

        // output = 1.0 * 0.5 + 0.5 * 0.5 = 0.5 + 0.25 = 0.75
        assert!((vca.sample(0.0) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_vca_with_zero_carrier() {
        let carrier = Constant::zero();
        let amplitude = Constant::new(0.5);
        let vca = VcaCentered::new(carrier, amplitude);

        // output = 0.0 * 0.5 + 0.5 * 0.5 = 0.25
        assert!((vca.sample(0.0) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_vca_output_range() {
        let carrier = Sine::with_frequency(1.0);
        let amplitude = Constant::new(0.5);
        let vca = VcaCentered::new(carrier, amplitude);

        for i in 0..100 {
            let t = i as f64 * 0.01;
            let v = vca.sample(t);
            assert!((0.0..=1.0).contains(&v), "Value {} out of range at t={}", v, t);
        }
    }

    #[test]
    fn test_vca_with_context() {
        let carrier = Constant::new(0.8);
        let amplitude = Constant::one();
        let vca = VcaCentered::new(carrier, amplitude);
        let ctx = SignalContext::default();

        assert!((vca.sample_with_context(0.0, &ctx) - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_vca_nan_carrier() {
        // Use a custom NaN signal to bypass Constant's sanitization
        struct NanSignal;
        impl Signal for NanSignal {
            fn sample(&self, _t: SignalTime) -> f32 {
                f32::NAN
            }
        }

        let vca = VcaCentered::new(NanSignal, Constant::one());
        let v = vca.sample(0.0);
        assert!(v.is_finite());
        assert_eq!(v, 0.5); // NaN carrier treated as 0.5
    }

    #[test]
    fn test_vca_nan_amplitude() {
        let carrier = Constant::new(0.8);
        // Simulate NaN amplitude
        struct NanSignal;
        impl Signal for NanSignal {
            fn sample(&self, _t: SignalTime) -> f32 {
                f32::NAN
            }
        }

        let vca = VcaCentered::new(carrier, NanSignal);
        let v = vca.sample(0.0);
        assert!(v.is_finite());
        assert_eq!(v, 0.5); // NaN amplitude treated as 0.0 → output = 0.5
    }
}

// <FILE>mixed-signals/src/composition/cls_vca_centered.rs</FILE> - <DESC>Centered voltage-controlled amplifier</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
