// <FILE>mixed-signals/src/generators/cls_constant.rs</FILE> - <DESC>Constant value signal</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Removed clamping, added output_range()</CLOG>

use crate::math::finite_or;
use crate::traits::{Signal, SignalRange, SignalTime};
use serde::{Deserialize, Serialize};

/// Constant signal that always returns the same value.
///
/// Useful as a baseline, for mixing, or for holding a fixed parameter.
///
/// **Note:** Output is NOT clamped. Use `.normalized()` to get [0, 1] range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Constant {
    pub value: f32,
}

impl Constant {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn zero() -> Self {
        Self { value: 0.0 }
    }

    pub fn one() -> Self {
        Self { value: 1.0 }
    }
}

impl Default for Constant {
    fn default() -> Self {
        Self { value: 0.0 }
    }
}

impl Signal for Constant {
    fn output_range(&self) -> SignalRange {
        let value = finite_or(self.value, 0.0);
        SignalRange::new(value, value)
    }

    fn sample(&self, _t: SignalTime) -> f32 {
        finite_or(self.value, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_value() {
        let c = Constant::new(0.42);
        assert!((c.sample(0.0) - 0.42).abs() < 0.001);
        assert!((c.sample(100.0) - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_constant_zero() {
        let c = Constant::zero();
        assert!((c.sample(0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_constant_one() {
        let c = Constant::one();
        assert!((c.sample(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_constant_no_clamp() {
        // Constant no longer clamps - values pass through
        let high = Constant::new(2.0);
        let low = Constant::new(-1.0);
        assert!((high.sample(0.0) - 2.0).abs() < 0.001);
        assert!((low.sample(0.0) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_constant_output_range() {
        let c = Constant::new(0.5);
        let range = c.output_range();
        assert_eq!(range.min, 0.5);
        assert_eq!(range.max, 0.5);
    }
}

// <FILE>mixed-signals/src/generators/cls_constant.rs</FILE> - <DESC>Constant value signal</DESC>
// <VERS>END OF VERSION: 1.1.0 - 2025-12-27</VERS>
