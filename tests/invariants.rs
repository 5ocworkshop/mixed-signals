// <FILE>tests/invariants.rs</FILE> - <DESC>Property-based tests for signal invariants</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Math robustness audit</WCTX>
// <CLOG>Initial PBT suite checking boundedness, range adherence, and determinism</CLOG>

//! Property-based tests verifying core Signal trait invariants.
//!
//! These tests use proptest to generate thousands of random parameter combinations
//! and verify that mathematical invariants hold for all of them:
//!
//! 1. **Boundedness**: Output is always finite (no NaN/Inf)
//! 2. **Range Adherence**: Output stays within declared output_range()
//! 3. **Determinism**: Same input always produces same output
//! 4. **Normalization**: .normalized() always produces [0, 1]

use mixed_signals::prelude::*;
use proptest::prelude::*;

// ============================================================================
// Strategies (Input Generators)
// ============================================================================

/// Strategy to generate valid finite f32s for signal parameters.
/// Constrained to reasonable values to avoid overflow in intermediate calculations.
fn finite_f32() -> impl Strategy<Value = f32> {
    -1e6f32..1e6f32
}

/// Strategy for practical time values (t >= 0, bounded).
/// Signals are designed for real-time use, not astronomical time scales.
fn positive_time() -> impl Strategy<Value = f64> {
    0.0f64..1e6
}

/// Strategy to generate a valid Sine wave
fn sine_strategy() -> impl Strategy<Value = Sine> {
    (finite_f32(), finite_f32(), finite_f32(), finite_f32())
        .prop_map(|(freq, amp, offset, phase)| Sine::new(freq, amp, offset, phase))
}

/// Strategy to generate a valid Square wave
fn square_strategy() -> impl Strategy<Value = Square> {
    (
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
        0.0f32..=1.0f32,
    )
        .prop_map(|(freq, amp, offset, phase, duty)| Square::new(freq, amp, offset, phase, duty))
}

/// Strategy to generate a valid Triangle wave
fn triangle_strategy() -> impl Strategy<Value = Triangle> {
    (finite_f32(), finite_f32(), finite_f32(), finite_f32())
        .prop_map(|(freq, amp, offset, phase)| Triangle::new(freq, amp, offset, phase))
}

/// Strategy to generate a valid Sawtooth wave
fn sawtooth_strategy() -> impl Strategy<Value = Sawtooth> {
    (
        finite_f32(),
        finite_f32(),
        finite_f32(),
        finite_f32(),
        any::<bool>(),
    )
        .prop_map(|(freq, amp, offset, phase, inverted)| {
            Sawtooth::new(freq, amp, offset, phase, inverted)
        })
}

/// Strategy to generate a valid Constant
fn constant_strategy() -> impl Strategy<Value = Constant> {
    finite_f32().prop_map(Constant::new)
}

/// Strategy to generate a valid WhiteNoise
fn white_noise_strategy() -> impl Strategy<Value = WhiteNoise> {
    (any::<u64>(), finite_f32(), 1.0f32..1000.0f32)
        .prop_map(|(seed, amp, sample_rate)| WhiteNoise::new(seed, amp, sample_rate))
}

// ============================================================================
// Invariant 1: Boundedness
// For ANY valid configuration and ANY finite time 't',
// the output must be finite (no NaN, no Inf).
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn sine_is_always_finite(signal in sine_strategy(), t in positive_time()) {
        let val = signal.sample(t);
        prop_assert!(val.is_finite(), "Sine produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn square_is_always_finite(signal in square_strategy(), t in positive_time()) {
        let val = signal.sample(t);
        prop_assert!(val.is_finite(), "Square produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn triangle_is_always_finite(signal in triangle_strategy(), t in positive_time()) {
        let val = signal.sample(t);
        prop_assert!(val.is_finite(), "Triangle produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn sawtooth_is_always_finite(signal in sawtooth_strategy(), t in positive_time()) {
        let val = signal.sample(t);
        prop_assert!(val.is_finite(), "Sawtooth produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn constant_is_always_finite(signal in constant_strategy(), t in positive_time()) {
        let val = signal.sample(t);
        prop_assert!(val.is_finite(), "Constant produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn white_noise_is_always_finite(signal in white_noise_strategy(), t in positive_time()) {
        let val = signal.sample(t);
        prop_assert!(val.is_finite(), "WhiteNoise produced non-finite value {} at t={}", val, t);
    }
}

// ============================================================================
// Invariant 2: Range Adherence
// The sampled value must ALWAYS fall within the declared output_range().
// We allow a small epsilon for floating point errors.
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn sine_respects_output_range(signal in sine_strategy(), t in positive_time()) {
        let range = signal.output_range();
        let val = signal.sample(t);

        // Skip if range is degenerate (can happen with zero amplitude)
        if !range.min.is_finite() || !range.max.is_finite() {
            return Ok(());
        }

        let eps = 1e-4;
        prop_assert!(
            val >= range.min - eps,
            "Sine value {} below min {} at t={}",
            val, range.min, t
        );
        prop_assert!(
            val <= range.max + eps,
            "Sine value {} above max {} at t={}",
            val, range.max, t
        );
    }

    #[test]
    fn square_respects_output_range(signal in square_strategy(), t in positive_time()) {
        let range = signal.output_range();
        let val = signal.sample(t);

        if !range.min.is_finite() || !range.max.is_finite() {
            return Ok(());
        }

        let eps = 1e-4;
        prop_assert!(val >= range.min - eps, "Square value {} below min {}", val, range.min);
        prop_assert!(val <= range.max + eps, "Square value {} above max {}", val, range.max);
    }

    #[test]
    fn triangle_respects_output_range(signal in triangle_strategy(), t in positive_time()) {
        let range = signal.output_range();
        let val = signal.sample(t);

        if !range.min.is_finite() || !range.max.is_finite() {
            return Ok(());
        }

        let eps = 1e-4;
        prop_assert!(val >= range.min - eps, "Triangle value {} below min {}", val, range.min);
        prop_assert!(val <= range.max + eps, "Triangle value {} above max {}", val, range.max);
    }

    #[test]
    fn sawtooth_respects_output_range(signal in sawtooth_strategy(), t in positive_time()) {
        let range = signal.output_range();
        let val = signal.sample(t);

        if !range.min.is_finite() || !range.max.is_finite() {
            return Ok(());
        }

        let eps = 1e-4;
        prop_assert!(val >= range.min - eps, "Sawtooth value {} below min {}", val, range.min);
        prop_assert!(val <= range.max + eps, "Sawtooth value {} above max {}", val, range.max);
    }
}

// ============================================================================
// Invariant 3: Determinism
// Calling sample(t) multiple times must yield bitwise identical results.
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn sine_is_deterministic(signal in sine_strategy(), t in positive_time()) {
        let v1 = signal.sample(t);
        let v2 = signal.sample(t);
        prop_assert_eq!(v1, v2, "Sine not deterministic at t={}", t);
    }

    #[test]
    fn square_is_deterministic(signal in square_strategy(), t in positive_time()) {
        let v1 = signal.sample(t);
        let v2 = signal.sample(t);
        prop_assert_eq!(v1, v2, "Square not deterministic at t={}", t);
    }

    #[test]
    fn triangle_is_deterministic(signal in triangle_strategy(), t in positive_time()) {
        let v1 = signal.sample(t);
        let v2 = signal.sample(t);
        prop_assert_eq!(v1, v2, "Triangle not deterministic at t={}", t);
    }

    #[test]
    fn noise_is_deterministic(seed in any::<u64>(), t in positive_time()) {
        let noise = WhiteNoise::new(seed, 1.0, 60.0);
        let v1 = noise.sample(t);
        let v2 = noise.sample(t);
        prop_assert_eq!(v1, v2, "WhiteNoise not deterministic at t={}", t);
    }

    #[test]
    fn perlin_is_deterministic(seed in any::<u64>(), t in positive_time()) {
        let noise = PerlinNoise::new(seed, 1.0, 1.0);
        let v1 = noise.sample(t);
        let v2 = noise.sample(t);
        prop_assert_eq!(v1, v2, "PerlinNoise not deterministic at t={}", t);
    }
}

// ============================================================================
// Invariant 4: Normalization
// The .normalized() wrapper must ALWAYS produce values in [0, 1]
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn normalized_sine_is_unit_range(signal in sine_strategy(), t in positive_time()) {
        let normalized = signal.normalized();
        let val = normalized.sample(t);

        // Must be finite
        prop_assert!(val.is_finite(), "Normalized sine produced non-finite value {}", val);

        let eps = 1e-4;
        prop_assert!(val >= 0.0 - eps, "Normalized value {} < 0", val);
        prop_assert!(val <= 1.0 + eps, "Normalized value {} > 1", val);
    }

    #[test]
    fn normalized_square_is_unit_range(signal in square_strategy(), t in positive_time()) {
        let normalized = signal.normalized();
        let val = normalized.sample(t);

        prop_assert!(val.is_finite(), "Normalized square produced non-finite value {}", val);

        let eps = 1e-4;
        prop_assert!(val >= 0.0 - eps, "Normalized value {} < 0", val);
        prop_assert!(val <= 1.0 + eps, "Normalized value {} > 1", val);
    }

    #[test]
    fn normalized_triangle_is_unit_range(signal in triangle_strategy(), t in positive_time()) {
        let normalized = signal.normalized();
        let val = normalized.sample(t);

        prop_assert!(val.is_finite(), "Normalized triangle produced non-finite value {}", val);

        let eps = 1e-4;
        prop_assert!(val >= 0.0 - eps, "Normalized value {} < 0", val);
        prop_assert!(val <= 1.0 + eps, "Normalized value {} > 1", val);
    }

    #[test]
    fn normalized_noise_is_unit_range(signal in white_noise_strategy(), t in positive_time()) {
        let normalized = signal.normalized();
        let val = normalized.sample(t);

        prop_assert!(val.is_finite(), "Normalized noise produced non-finite value {}", val);

        let eps = 1e-4;
        prop_assert!(val >= 0.0 - eps, "Normalized value {} < 0", val);
        prop_assert!(val <= 1.0 + eps, "Normalized value {} > 1", val);
    }
}

// ============================================================================
// Invariant 5: Composition Preserves Finiteness
// Composing finite signals should produce finite results
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn add_preserves_finiteness(
        a in sine_strategy(),
        b in triangle_strategy(),
        t in positive_time()
    ) {
        let composed = a.add(b);
        let val = composed.sample(t);
        prop_assert!(val.is_finite(), "Add produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn multiply_preserves_finiteness(
        a in sine_strategy(),
        b in constant_strategy(),
        t in positive_time()
    ) {
        let composed = a.multiply(b);
        let val = composed.sample(t);
        prop_assert!(val.is_finite(), "Multiply produced non-finite value {} at t={}", val, t);
    }

    #[test]
    fn mix_preserves_finiteness(
        a in sine_strategy(),
        b in triangle_strategy(),
        blend in 0.0f32..=1.0f32,
        t in positive_time()
    ) {
        let composed = a.mix(b, blend);
        let val = composed.sample(t);
        prop_assert!(val.is_finite(), "Mix produced non-finite value {} at t={}", val, t);
    }
}

// <FILE>tests/invariants.rs</FILE> - <DESC>Property-based tests for signal invariants</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
