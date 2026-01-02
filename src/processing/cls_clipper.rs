// <FILE>src/processing/cls_clipper.rs</FILE> - <DESC>Asymmetric clipper/limiter</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Audio synthesis - adding distortion/saturation</WCTX>
// <CLOG>Initial implementation - asymmetric hard/soft clipping</CLOG>

use crate::traits::{Signal, SignalContext, SignalTime};

/// Clipping mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipMode {
    /// Hard clipping - values beyond threshold are clamped
    Hard,
    /// Soft clipping - smooth saturation curve (tanh-like)
    Soft,
}

/// Asymmetric clipper/limiter for adding harmonics and saturation.
///
/// Clips the signal at configurable positive and negative thresholds.
/// Useful for:
/// - Adding subtle harmonics (soft clip at 0.7 adds warmth)
/// - Creating distortion effects
/// - Asymmetric waveform shaping
///
/// # Example
/// ```ignore
/// let sine = Sine::new(440.0, 1.0, 0.0, 0.0);
/// // Clip positive peaks at 0.7, leave negative untouched
/// let clipped = Clipper::asymmetric(sine, 0.7, -1.0);
/// ```
#[derive(Debug, Clone)]
pub struct Clipper<S> {
    signal: S,
    /// Maximum positive value (clips above this)
    pos_threshold: f32,
    /// Minimum negative value (clips below this)
    neg_threshold: f32,
    /// Clipping mode
    mode: ClipMode,
}

impl<S: Signal> Clipper<S> {
    /// Create a new clipper with asymmetric thresholds.
    ///
    /// # Arguments
    /// * `signal` - Input signal
    /// * `pos_threshold` - Maximum positive value (e.g., 0.7)
    /// * `neg_threshold` - Minimum negative value (e.g., -1.0)
    /// * `mode` - Hard or soft clipping
    pub fn new(signal: S, pos_threshold: f32, neg_threshold: f32, mode: ClipMode) -> Self {
        Self {
            signal,
            pos_threshold,
            neg_threshold,
            mode,
        }
    }

    /// Create a hard clipper with asymmetric thresholds.
    pub fn asymmetric(signal: S, pos_threshold: f32, neg_threshold: f32) -> Self {
        Self::new(signal, pos_threshold, neg_threshold, ClipMode::Hard)
    }

    /// Create a symmetric hard clipper.
    pub fn symmetric(signal: S, threshold: f32) -> Self {
        Self::new(signal, threshold, -threshold, ClipMode::Hard)
    }

    /// Create a soft clipper with asymmetric thresholds.
    pub fn soft(signal: S, pos_threshold: f32, neg_threshold: f32) -> Self {
        Self::new(signal, pos_threshold, neg_threshold, ClipMode::Soft)
    }

    /// Create a symmetric soft clipper.
    pub fn soft_symmetric(signal: S, threshold: f32) -> Self {
        Self::new(signal, threshold, -threshold, ClipMode::Soft)
    }

    fn clip(&self, value: f32) -> f32 {
        match self.mode {
            ClipMode::Hard => value.clamp(self.neg_threshold, self.pos_threshold),
            ClipMode::Soft => self.soft_clip(value),
        }
    }

    fn soft_clip(&self, value: f32) -> f32 {
        // Soft clipping using a piecewise approach
        // Below threshold: linear
        // Above threshold: smooth saturation curve
        if value > self.pos_threshold {
            // Positive soft saturation
            let excess = value - self.pos_threshold;
            let headroom = 1.0 - self.pos_threshold;
            if headroom > 0.0 {
                self.pos_threshold + headroom * (1.0 - (-excess / headroom).exp())
            } else {
                self.pos_threshold
            }
        } else if value < self.neg_threshold {
            // Negative soft saturation
            let excess = self.neg_threshold - value;
            let headroom = 1.0 + self.neg_threshold; // Distance from -1
            if headroom > 0.0 {
                self.neg_threshold - headroom * (1.0 - (-excess / headroom).exp())
            } else {
                self.neg_threshold
            }
        } else {
            // Linear region
            value
        }
    }
}

impl<S: Signal> Signal for Clipper<S> {
    fn sample(&self, t: SignalTime) -> f32 {
        let input = self.signal.sample(t);
        self.clip(input)
    }

    fn sample_with_context(&self, t: SignalTime, ctx: &SignalContext) -> f32 {
        let input = self.signal.sample_with_context(t, ctx);
        self.clip(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::SignalTime;

    /// Test signal that returns a raw value without clamping (for bipolar tests)
    struct RawSignal(f32);
    impl Signal for RawSignal {
        fn sample(&self, _t: SignalTime) -> f32 {
            self.0
        }
    }

    #[test]
    fn test_clipper_hard_symmetric() {
        let sig = RawSignal(0.9);
        let clipped = Clipper::symmetric(sig, 0.7);

        let result = clipped.sample(0.0);
        assert!(
            (result - 0.7).abs() < 0.001,
            "Should clip at 0.7: {}",
            result
        );
    }

    #[test]
    fn test_clipper_hard_asymmetric() {
        // Positive threshold lower than negative
        let sig_pos = RawSignal(0.9);
        let clipped_pos = Clipper::asymmetric(sig_pos, 0.7, -1.0);
        assert!((clipped_pos.sample(0.0) - 0.7).abs() < 0.001);

        // Negative value should pass through
        let sig_neg = RawSignal(-0.9);
        let clipped_neg = Clipper::asymmetric(sig_neg, 0.7, -1.0);
        assert!((clipped_neg.sample(0.0) - (-0.9)).abs() < 0.001);
    }

    #[test]
    fn test_clipper_passthrough() {
        let sig = RawSignal(0.5);
        let clipped = Clipper::symmetric(sig, 0.7);

        let result = clipped.sample(0.0);
        assert!(
            (result - 0.5).abs() < 0.001,
            "Should pass through: {}",
            result
        );
    }

    #[test]
    fn test_clipper_soft_saturation() {
        let sig = RawSignal(1.5);
        let clipped = Clipper::soft_symmetric(sig, 0.7);

        let result = clipped.sample(0.0);
        // Soft clip should be between threshold and 1.0
        assert!(
            result > 0.7,
            "Soft clip should exceed threshold: {}",
            result
        );
        assert!(result < 1.0, "Soft clip should be below 1.0: {}", result);
    }

    #[test]
    fn test_clipper_soft_smooth() {
        // Test that soft clipping is continuous
        let threshold = 0.7;
        let just_below = RawSignal(threshold - 0.01);
        let just_above = RawSignal(threshold + 0.01);

        let clip_below = Clipper::soft_symmetric(just_below, threshold);
        let clip_above = Clipper::soft_symmetric(just_above, threshold);

        let below = clip_below.sample(0.0);
        let above = clip_above.sample(0.0);

        // Values should be close (smooth transition)
        assert!(
            (above - below).abs() < 0.05,
            "Should be smooth: {} vs {}",
            below,
            above
        );
    }

    #[test]
    fn test_clipper_negative_soft() {
        let sig = RawSignal(-1.5);
        let clipped = Clipper::soft_symmetric(sig, 0.7);

        let result = clipped.sample(0.0);
        assert!(result < -0.7, "Soft clip should be below -0.7: {}", result);
        assert!(result > -1.0, "Soft clip should be above -1.0: {}", result);
    }

    #[test]
    fn test_clipper_output_finite() {
        let sig = RawSignal(100.0); // Way beyond threshold
        let clipped = Clipper::soft_symmetric(sig, 0.7);

        let result = clipped.sample(0.0);
        assert!(result.is_finite(), "Should be finite: {}", result);
        assert!(result <= 1.0, "Should not exceed 1.0: {}", result);
    }
}

// <FILE>src/processing/cls_clipper.rs</FILE> - <DESC>Asymmetric clipper/limiter</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
