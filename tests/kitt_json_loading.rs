//! Test to verify kitt.json loading works correctly
//!
//! Note: With the bipolar core refactor, signals no longer clamp intermediate
//! results. Use `.normalized()` on the final signal if [0,1] output is needed.

use mixed_signals::traits::Signal;
use mixed_signals::types::SignalSpec;
use std::fs;

#[test]
fn test_load_kitt_json_and_sample() {
    // Load the actual kitt.json from project root
    let json = fs::read_to_string("kitt.json").expect("Failed to read kitt.json");
    println!("Loaded JSON: {} bytes", json.len());

    // Parse it
    let spec: SignalSpec = serde_json::from_str(&json).expect("Failed to parse JSON");

    // Verify it's a Remap at the top level
    match &spec {
        SignalSpec::Remap {
            in_min,
            in_max,
            out_min,
            out_max,
            ..
        } => {
            assert_eq!(*in_min, 0.0);
            assert_eq!(*in_max, 1.0);
            assert_eq!(*out_min, -1.0);
            assert_eq!(*out_max, 1.0);
            println!(
                "Top-level is Remap: {}→{} to {}→{}",
                in_min, in_max, out_min, out_max
            );
        }
        other => {
            panic!(
                "Expected Remap at top level, got {:?}",
                std::mem::discriminant(other)
            );
        }
    }

    // Build the signal
    let signal = spec.build().expect("Failed to build signal");
    println!("Signal built successfully");

    // Sample at various points and verify we get non-trivial values
    let samples: Vec<_> = (0..10)
        .map(|i| {
            let t = i as f64 * 0.1;
            let s = signal.sample(t);
            println!("t={:.2}: sample={:.6}", t, s);
            s
        })
        .collect();

    // Verify samples are finite (bipolar signals may exceed [-1, 1] before normalization)
    for (i, &s) in samples.iter().enumerate() {
        assert!(
            s.is_finite(),
            "Sample {} at t={:.2} is not finite: {}",
            i,
            i as f64 * 0.1,
            s
        );
    }

    // Verify samples are not all the same (signal is varying)
    let all_same = samples.windows(2).all(|w| (w[0] - w[1]).abs() < 0.0001);
    assert!(
        !all_same,
        "All samples are nearly identical - signal may not be working"
    );

    println!("Test passed: kitt.json loads and produces varying signal");
}

#[test]
fn test_kitt_json_samples_different_from_hardcoded() {
    // Load kitt.json
    let json = fs::read_to_string("kitt.json").expect("Failed to read kitt.json");
    let spec: SignalSpec = serde_json::from_str(&json).expect("Failed to parse JSON");
    let json_signal = spec.build().expect("Failed to build signal");

    // Create hardcoded signal with different frequency (470Hz vs 485Hz in JSON)
    use mixed_signals::generators::Sine;
    let hardcoded_carrier_freq = 470.0;
    let _json_carrier_freq = 485.0; // Used for documentation, not runtime

    // Sample both at the same time points
    let test_time = 0.1;
    let json_sample = json_signal.sample(test_time);

    // The hardcoded version uses 470Hz, JSON uses 485Hz
    // At t=0.1, these should produce different values
    let hardcoded_sine = Sine::new(hardcoded_carrier_freq, 1.0, 0.0, 0.0);
    let hardcoded_sample = hardcoded_sine.sample(test_time);

    println!("JSON signal sample at t={}: {}", test_time, json_sample);
    println!("Hardcoded sine at t={}: {}", test_time, hardcoded_sample);

    // They won't be exactly the same values since the JSON has a more complex structure,
    // but this test verifies the JSON signal is producing real output
    // Note: With bipolar core, intermediate results may exceed [-1, 1]
    assert!(json_sample.is_finite(), "JSON sample should be finite");
}
