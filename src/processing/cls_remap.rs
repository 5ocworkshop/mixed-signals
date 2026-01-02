// <FILE>mixed-signals/src/processing/cls_remap.rs</FILE> - <DESC>Signal range remapping operator</DESC>
// <VERS>VERSION: 1.0.0 - 2025-12-22</VERS>
// <WCTX>Signal generator implementation</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::traits::{Signal, SignalContext, SignalRange, SignalTime};

/// Remaps a signal from one range to another.
///
/// Linearly maps values from [in_min, in_max] to [out_min, out_max].
#[derive(Debug, Clone)]
pub struct Remap<S> {
    pub signal: S,
    pub in_min: f32,
    pub in_max: f32,
    pub out_min: f32,
    pub out_max: f32,
}

impl<S: Signal> Remap<S> {
    pub fn new(signal: S, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> Self {
        Self {
            signal,
            in_min,
            in_max,
            out_min,
            out_max,
        }
    }

    /// Remap from -1..1 to 0..1
    pub fn to_unit(signal: S) -> Self {
        Self::new(signal, -1.0, 1.0, 0.0, 1.0)
    }

    /// Remap from 0..1 to -1..1
    pub fn to_bipolar(signal: S) -> Self {
        Self::new(signal, 0.0, 1.0, -1.0, 1.0)
    }
}

impl<S: Signal> Signal for Remap<S> {
    fn output_range(&self) -> SignalRange {
        SignalRange::new(self.out_min, self.out_max)
    }

    fn sample(&self, t: SignalTime) -> f32 {
        let v = self.signal.sample(t);
        if !self.in_min.is_finite()
            || !self.in_max.is_finite()
            || !self.out_min.is_finite()
            || !self.out_max.is_finite()
        {
            return v;
        }
        let in_range = self.in_max - self.in_min;
        let out_range = self.out_max - self.out_min;

        if in_range.abs() < 0.0001 {
            return self.out_min;
        }

        let normalized = (v - self.in_min) / in_range;
        self.out_min + normalized * out_range
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let v = self.signal.sample_with_context(t, ctx);
        if !self.in_min.is_finite()
            || !self.in_max.is_finite()
            || !self.out_min.is_finite()
            || !self.out_max.is_finite()
        {
            return v;
        }
        let in_range = self.in_max - self.in_min;
        let out_range = self.out_max - self.out_min;

        if in_range.abs() < 0.0001 {
            return self.out_min;
        }

        let normalized = (v - self.in_min) / in_range;
        self.out_min + normalized * out_range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Constant;

    struct RawSignal(f32);

    impl Signal for RawSignal {
        fn sample(&self, _t: SignalTime) -> f32 {
            self.0
        }
    }

    #[test]
    fn test_remap_identity() {
        let sig = Constant::new(0.5);
        let remapped = Remap::new(sig, 0.0, 1.0, 0.0, 1.0);
        assert!((remapped.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_remap_to_unit() {
        let sig = RawSignal(-1.0);
        let remapped = Remap::to_unit(sig);
        assert!((remapped.sample(0.0) - 0.0).abs() < 0.001);

        let sig2 = Constant::new(1.0);
        let remapped2 = Remap::to_unit(sig2);
        assert!((remapped2.sample(0.0) - 1.0).abs() < 0.001);

        let sig3 = RawSignal(0.0);
        let remapped3 = Remap::to_unit(sig3);
        assert!((remapped3.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_remap_to_bipolar() {
        let sig = Constant::new(0.0);
        let remapped = Remap::to_bipolar(sig);
        assert!((remapped.sample(0.0) - -1.0).abs() < 0.001);

        let sig2 = Constant::new(1.0);
        let remapped2 = Remap::to_bipolar(sig2);
        assert!((remapped2.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_remap_custom() {
        let sig = RawSignal(50.0);
        let remapped = Remap::new(sig, 0.0, 100.0, 0.0, 1.0);
        assert!((remapped.sample(0.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_remap_non_finite_bounds_falls_back() {
        let sig = RawSignal(0.7);
        let remapped = Remap::new(sig, f32::NAN, 1.0, 0.0, 1.0);
        assert!((remapped.sample(0.0) - 0.7).abs() < 0.001);
    }
}

// <FILE>mixed-signals/src/processing/cls_remap.rs</FILE> - <DESC>Signal range remapping operator</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
