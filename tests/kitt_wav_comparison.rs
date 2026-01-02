//! Test to compare JSON vs hardcoded WAV output

use mixed_signals::prelude::*;
use mixed_signals::traits::Signal;
use mixed_signals::traits::{Fn1, SignalExt};
use mixed_signals::types::SignalSpec;
use std::fs;

fn sample_json_signal() -> Vec<f32> {
    let json = fs::read_to_string("kitt.json").expect("Failed to read kitt.json");
    let spec: SignalSpec = serde_json::from_str(&json).expect("Failed to parse JSON");
    let signal = spec.build().expect("Failed to build signal");

    let duration = 1.1757;
    let sample_rate = 48000.0;
    let num_samples = (duration * sample_rate) as usize;
    let dt = 1.0 / sample_rate;

    (0..num_samples)
        .map(|i| {
            let t = i as f64 * dt;
            signal.sample(t).clamp(-1.0, 1.0)
        })
        .collect()
}

fn sample_hardcoded_signal() -> Vec<f32> {
    let duration = 1.1757;
    let sample_rate = 48000.0;
    let num_samples = (duration * sample_rate) as usize;
    let dt = 1.0 / sample_rate;

    // Hardcoded parameters (from kitt.rs)
    let f_mod: f32 = 2.47;
    let f_car: f32 = 470.0;
    let dev_hz: f32 = 150.0;
    let depth = dev_hz / f_mod;
    let phase_fm: f32 = 0.6913;
    let phase_amp: f32 = 0.9413;
    let tail_width: f64 = 6.0;
    let norm_tail = ((tail_width - 6.0) / (20.0 - 6.0)).clamp(0.0, 1.0);
    let release = (0.45 - (norm_tail * 0.25)) as f32;

    let carrier =
        Sine::new(f_car, 1.0, 0.0, 0.0).mix(Sawtooth::new(f_car, 1.0, 0.0, 0.0, false), 0.25);
    let vibrato = FrequencyMod::new(carrier, Sine::new(f_mod, 1.0, 0.0, phase_fm), depth, f_car);
    let macro_env = Fn1(move |t: f64| {
        let p = (t / duration).clamp(0.0, 1.0);
        LinearEnvelope::new(0.10, release).sample(p)
    })
    .map(|v| v.powf(2.2));
    let tremolo = Sine::new(f_mod, 0.65, 0.35, phase_amp);
    let textured = vibrato.mix(PinkNoise::new(42, 1.0, 0.0), 0.08);
    let final_signal = textured.multiply(macro_env).multiply(tremolo).scale(0.95);
    let audio_out = Remap::to_bipolar(final_signal);

    (0..num_samples)
        .map(|i| {
            let t = i as f64 * dt;
            audio_out.sample(t)
        })
        .collect()
}

#[test]
fn test_json_and_hardcoded_produce_different_output() {
    let json_samples = sample_json_signal();
    let hardcoded_samples = sample_hardcoded_signal();

    assert_eq!(json_samples.len(), hardcoded_samples.len());

    // Print first few samples from each
    println!("\nFirst 10 samples comparison:");
    println!(
        "{:>10} {:>12} {:>12} {:>12}",
        "Index", "JSON", "Hardcoded", "Diff"
    );
    for i in 0..10 {
        let json_s = json_samples[i];
        let hard_s = hardcoded_samples[i];
        let diff = (json_s - hard_s).abs();
        println!("{:>10} {:>12.6} {:>12.6} {:>12.6}", i, json_s, hard_s, diff);
    }

    // Sample at key time points
    let key_indices: Vec<usize> = vec![0, 4800, 9600, 24000, 48000]; // t=0, 0.1, 0.2, 0.5, 1.0
    println!("\nKey time points:");
    println!(
        "{:>10} {:>12} {:>12} {:>12}",
        "Time(s)", "JSON", "Hardcoded", "Diff"
    );
    for &idx in &key_indices {
        if idx < json_samples.len() {
            let t = idx as f64 / 48000.0;
            let json_s = json_samples[idx];
            let hard_s = hardcoded_samples[idx];
            let diff = (json_s - hard_s).abs();
            println!(
                "{:>10.3} {:>12.6} {:>12.6} {:>12.6}",
                t, json_s, hard_s, diff
            );
        }
    }

    // Calculate total difference
    let total_diff: f64 = json_samples
        .iter()
        .zip(hardcoded_samples.iter())
        .map(|(j, h)| (*j as f64 - *h as f64).abs())
        .sum();
    let avg_diff = total_diff / json_samples.len() as f64;
    println!("\nAverage sample difference: {:.6}", avg_diff);

    // They should be different
    assert!(
        avg_diff > 0.01,
        "JSON and hardcoded should produce different output, avg_diff={}",
        avg_diff
    );
    println!("\nConfirmed: JSON and hardcoded produce DIFFERENT output");
}
