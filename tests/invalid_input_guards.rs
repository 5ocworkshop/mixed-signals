use mixed_signals::envelopes::{Adsr, Impact, LinearEnvelope};
use mixed_signals::generators::{Constant, Pulse, Ramp, Sawtooth, Sine, Square, Step, Triangle};
use mixed_signals::noise::{PerlinNoise, WhiteNoise};
use mixed_signals::random::{PerCharacterNoise, PinkNoise, SeededRandom, SpatialNoise};
use mixed_signals::traits::{Signal, SignalExt};

fn assert_finite(value: f32) {
    assert!(value.is_finite(), "Value must be finite, got {}", value);
}

fn assert_unit(value: f32) {
    assert!(value.is_finite(), "Value must be finite, got {}", value);
    assert!((0.0..=1.0).contains(&value), "Value {} out of 0..1", value);
}

#[test]
fn test_invalid_inputs_generators_are_finite() {
    // Generators produce finite values even with invalid inputs
    let sine = Sine::new(f32::NAN, f32::INFINITY, f32::NAN, f32::INFINITY);
    assert_finite(sine.sample(f64::NAN));

    let triangle = Triangle::new(f32::NAN, f32::INFINITY, f32::NAN, f32::INFINITY);
    assert_finite(triangle.sample(f64::NAN));

    let square = Square::new(f32::NAN, f32::INFINITY, f32::NAN, f32::INFINITY, f32::NAN);
    assert_finite(square.sample(f64::NAN));

    let saw = Sawtooth::new(f32::NAN, f32::INFINITY, f32::NAN, f32::INFINITY, false);
    assert_finite(saw.sample(f64::NAN));

    let ramp = Ramp::new(f32::NAN, f32::INFINITY, f32::NAN);
    assert_finite(ramp.sample(f64::NAN));

    let step = Step::new(f32::NAN, f32::INFINITY, f32::NAN);
    assert_finite(step.sample(f64::NAN));

    let pulse = Pulse::new(f32::NAN, f32::INFINITY, f32::NAN, f32::NAN);
    assert_finite(pulse.sample(f64::NAN));

    let constant = Constant::new(f32::NAN);
    assert_finite(constant.sample(f64::NAN));
}

#[test]
fn test_generators_normalized_are_unit() {
    // .normalized() produces 0..1 values even from bipolar signals
    let sine = Sine::default().normalized();
    assert_unit(sine.sample(0.0));
    assert_unit(sine.sample(0.25));
    assert_unit(sine.sample(0.75));

    let triangle = Triangle::default().normalized();
    assert_unit(triangle.sample(0.0));
    assert_unit(triangle.sample(0.5));

    let square = Square::default().normalized();
    assert_unit(square.sample(0.0));
    assert_unit(square.sample(0.75));

    let saw = Sawtooth::default().normalized();
    assert_unit(saw.sample(0.0));
    assert_unit(saw.sample(0.99));
}

#[test]
fn test_invalid_inputs_noise_random_are_finite() {
    // Noise generators with NAN params produce finite values using defaults
    // Note: These now output bipolar [-1, 1], use .normalized() for [0, 1]
    let white = WhiteNoise::new(1, f32::NAN, f32::NAN);
    assert_unit(white.normalized().sample(f64::NAN));

    let perlin = PerlinNoise::new(1, f32::NAN, f32::NAN);
    assert_unit(perlin.normalized().sample(f64::NAN));

    let seeded = SeededRandom::new(1, f32::NAN, f32::NAN);
    assert_unit(seeded.normalized().sample(f64::NAN));

    let spatial = SpatialNoise::new(1, f32::NAN, f32::NAN);
    assert_unit(spatial.normalized().sample(f64::NAN));

    let pink = PinkNoise::new(1, f32::NAN, f32::NAN);
    assert_unit(pink.normalized().sample(f64::NAN));

    let per_char = PerCharacterNoise::new(1, f32::NAN, f32::NAN);
    assert_unit(per_char.normalized().sample(f64::NAN));
}

#[test]
fn test_invalid_inputs_envelopes_are_finite() {
    let adsr = Adsr::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN).with_peak(f32::NAN);
    assert_unit(adsr.sample(f64::NAN));

    let linear = LinearEnvelope::new(f32::NAN, f32::NAN).with_peak(f32::NAN);
    assert_unit(linear.sample(f64::NAN));

    let impact = Impact::new(f32::NAN, f32::NAN);
    assert_unit(impact.sample(f64::NAN));
}
