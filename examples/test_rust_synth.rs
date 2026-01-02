// <FILE>examples/test_rust_synth.rs</FILE> - <DESC>KITT scanner sound synthesis test</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Testing mixed-signals library synthesis capabilities</WCTX>
// <CLOG>Fixed double normalization and noise speckles</CLOG>

//! KITT Scanner Sound - Full mixed-signals Synthesis
//!
//! Implements the complete tone_only.py recipe using library primitives.
//!
//! Run with: cargo run --example test_rust_synth

use mixed_signals::generators::{Keyframe, Keyframes, Sine};
use mixed_signals::processing::Remap;
use mixed_signals::traits::Signal;
use std::f32::consts::TAU;

fn main() {
    let sample_rate = 48000u32;
    let duration = 1.176f64;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let dt = 1.0 / sample_rate as f64;
    let sr_f32 = sample_rate as f32;

    println!("Generating KITT scanner sound with FULL recipe...");
    println!(
        "Sample rate: {} Hz, Duration: {:.3}s, Samples: {}",
        sample_rate, duration, num_samples
    );

    // =========================================================================
    // Signal graph components
    // =========================================================================

    // Envelope keyframes (heartbeat shape from tone_only.py)
    let envelope = Keyframes::new(vec![
        Keyframe::new(0.00, 0.20),
        Keyframe::new(0.06, 0.22),
        Keyframe::new(0.10, 0.45),
        Keyframe::new(0.15, 0.90), // Peak 1 (S1/lub)
        Keyframe::new(0.17, 0.70),
        Keyframe::new(0.20, 0.55),
        Keyframe::new(0.25, 0.62),
        Keyframe::new(0.30, 0.60),
        Keyframe::new(0.35, 0.58),
        Keyframe::new(0.40, 0.60),
        Keyframe::new(0.45, 0.72),
        Keyframe::new(0.50, 1.00), // Peak 2 (S2/dub)
        Keyframe::new(0.53, 0.80),
        Keyframe::new(0.56, 0.55),
        Keyframe::new(0.60, 0.35),
        Keyframe::new(0.65, 0.18),
        Keyframe::new(0.70, 0.08),
        Keyframe::new(0.75, 0.03),
        Keyframe::new(0.85, 0.01),
        Keyframe::new(1.00, 0.00),
        Keyframe::new(1.20, 0.00),
    ]);

    // Frequency sweep keyframes (380-520 Hz)
    let freq_min = 380.0f32;
    let freq_range = 140.0f32; // 520 - 380
    let freq_keyframes = Keyframes::new(vec![
        Keyframe::new(0.00, freq_min + 0.20 * freq_range),
        Keyframe::new(0.06, freq_min + 0.22 * freq_range),
        Keyframe::new(0.10, freq_min + 0.45 * freq_range),
        Keyframe::new(0.15, freq_min + 0.90 * freq_range),
        Keyframe::new(0.17, freq_min + 0.70 * freq_range),
        Keyframe::new(0.20, freq_min + 0.55 * freq_range),
        Keyframe::new(0.25, freq_min + 0.62 * freq_range),
        Keyframe::new(0.30, freq_min + 0.60 * freq_range),
        Keyframe::new(0.35, freq_min + 0.58 * freq_range),
        Keyframe::new(0.40, freq_min + 0.60 * freq_range),
        Keyframe::new(0.45, freq_min + 0.72 * freq_range),
        Keyframe::new(0.50, freq_min + 1.00 * freq_range),
        Keyframe::new(0.53, freq_min + 0.80 * freq_range),
        Keyframe::new(0.56, freq_min + 0.55 * freq_range),
        Keyframe::new(0.60, freq_min + 0.35 * freq_range),
        Keyframe::new(0.65, freq_min + 0.18 * freq_range),
        Keyframe::new(0.70, freq_min + 0.08 * freq_range),
        Keyframe::new(0.75, freq_min + 0.03 * freq_range),
        Keyframe::new(0.85, freq_min + 0.01 * freq_range),
        Keyframe::new(1.00, freq_min),
        Keyframe::new(1.20, freq_min),
    ]);

    // Vibrato LFO: 8.33 Hz, ±3.25 Hz depth
    let vibrato = Sine::new(8.33, 1.0, 0.0, 0.0);
    let vibrato_mapped = Remap::new(vibrato, 0.0, 1.0, -3.25, 3.25);

    // Hard gate envelope (fade 0.70s → 1.00s)
    let gate = Keyframes::new(vec![
        Keyframe::new(0.00, 1.0),
        Keyframe::new(0.70, 1.0),
        Keyframe::new(1.00, 0.0),
        Keyframe::new(1.20, 0.0),
    ]);

    // =========================================================================
    // Parameters from tone_only.py
    // =========================================================================
    let beat_detune = 13.0f32; // Hz - quarter tone interval
    let sub_bass_level = 0.4f32; // Sub-bass amplitude
    let tone_level = 0.90f32; // Tone mix level
    let noise_level = 0.10f32; // Noise mix level

    // =========================================================================
    // Generate ALL signals in a SINGLE pass (fixes filter state issues)
    // =========================================================================

    // Phase accumulators for oscillators
    let mut phase_beat1 = 0.0f32;
    let mut phase_beat2 = 0.0f32;
    let mut phase_sub = 0.0f32;

    // Manual SVF state for noise (matches tone_only.py exactly)
    let mut svf_low = 0.0f32;
    let mut svf_band = 0.0f32;
    let svf_q = 15.0f32;

    // Biquad filter states for tone processing
    // Lowpass 480 Hz
    let (lp_b0, lp_b1, lp_b2, lp_a1, lp_a2) =
        biquad_lowpass_coeffs(480.0, std::f32::consts::FRAC_1_SQRT_2, sr_f32);
    let mut lp_x1 = 0.0f32;
    let mut lp_x2 = 0.0f32;
    let mut lp_y1 = 0.0f32;
    let mut lp_y2 = 0.0f32;

    // Notch 450 Hz, Q=2
    let (nt_b0, nt_b1, nt_b2, nt_a1, nt_a2) = biquad_notch_coeffs(450.0, 2.0, sr_f32);
    let mut nt_x1 = 0.0f32;
    let mut nt_x2 = 0.0f32;
    let mut nt_y1 = 0.0f32;
    let mut nt_y2 = 0.0f32;

    // Seeded RNG for reproducible noise
    let mut rng_state = 42u64;
    let mut next_noise = || -> f32 {
        // SplitMix64
        rng_state = rng_state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z = z ^ (z >> 31);
        // Convert to -1..1
        (z as f32 / u64::MAX as f32) * 2.0 - 1.0
    };

    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f64 * dt;
        let dt_f32 = dt as f32;

        // --- Envelope and frequency ---
        let env_val = envelope.sample(t);
        let base_freq = freq_keyframes.sample(t);
        let vib_offset = vibrato_mapped.sample(t);
        let modulated_freq = base_freq + vib_offset;

        // --- Oscillators ---
        let freq1 = modulated_freq + beat_detune;
        let freq2 = modulated_freq - beat_detune;
        let freq_sub = modulated_freq * 0.5;

        phase_beat1 += freq1 * dt_f32;
        phase_beat2 += freq2 * dt_f32;
        phase_sub += freq_sub * dt_f32;

        phase_beat1 = phase_beat1.fract();
        phase_beat2 = phase_beat2.fract();
        phase_sub = phase_sub.fract();

        let beat1 = (TAU * phase_beat1).sin();
        let beat2 = (TAU * phase_beat2).sin();
        let sub = (TAU * phase_sub).sin() * sub_bass_level;

        let raw_tone = beat1 + beat2 + sub;

        // --- Lowpass filter (480 Hz) ---
        let lp_out =
            lp_b0 * raw_tone + lp_b1 * lp_x1 + lp_b2 * lp_x2 - lp_a1 * lp_y1 - lp_a2 * lp_y2;
        lp_x2 = lp_x1;
        lp_x1 = raw_tone;
        lp_y2 = lp_y1;
        lp_y1 = lp_out;

        // --- Notch filter (450 Hz, Q=2) ---
        let nt_out = nt_b0 * lp_out + nt_b1 * nt_x1 + nt_b2 * nt_x2 - nt_a1 * nt_y1 - nt_a2 * nt_y2;
        nt_x2 = nt_x1;
        nt_x1 = lp_out;
        nt_y2 = nt_y1;
        nt_y1 = nt_out;

        // --- Clipper (asymmetric: +0.7, -1.0) ---
        let clipped = nt_out.clamp(-1.0, 0.7);

        // --- SVF bandpass for noise (Q=15, sweeping cutoff) ---
        let white = next_noise();
        let cutoff_hz = base_freq.clamp(20.0, sr_f32 * 0.49);
        let f = 2.0 * (std::f32::consts::PI * cutoff_hz / sr_f32).sin();
        let q_inv = 1.0 / svf_q;

        svf_low += f * svf_band;
        let svf_high = white - svf_low - q_inv * svf_band;
        svf_band += f * svf_high;
        let filtered_noise = svf_band; // Bandpass output

        // --- Mix ---
        let mixed = clipped * tone_level + filtered_noise * noise_level;

        // --- Apply envelope ---
        let enveloped = mixed * env_val;

        // --- Hard gate ---
        let gate_val = gate.sample(t);
        let output = enveloped * gate_val;

        samples.push(output);
    }

    // =========================================================================
    // Single normalization at the end (matching Python)
    // =========================================================================
    let max_val = samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    if max_val > 0.0 {
        let scale = 0.75 / max_val;
        for s in &mut samples {
            *s *= scale;
        }
    }

    // =========================================================================
    // Write WAV file
    // =========================================================================

    match write_wav("test_rust.wav", sample_rate, &samples) {
        Ok(_) => {
            println!("\nWritten: test_rust.wav");
            println!("  {} samples, {:.3}s duration", num_samples, duration);
            println!("\nFixes applied:");
            println!("  [✓] Single-pass processing (no filter state issues)");
            println!("  [✓] Single normalization at end (no compression)");
            println!("  [✓] Deterministic noise (seeded RNG)");
            println!("  [✓] Inline filter processing (proper state)");
        }
        Err(e) => eprintln!("Error writing WAV: {}", e),
    }
}

// Biquad coefficient calculators
fn biquad_lowpass_coeffs(cutoff_hz: f32, q: f32, sample_rate: f32) -> (f32, f32, f32, f32, f32) {
    use std::f32::consts::PI;
    let omega = 2.0 * PI * cutoff_hz / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    let b1 = 1.0 - cos_omega;
    let b0 = b1 / 2.0;
    let b2 = b0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

fn biquad_notch_coeffs(center_hz: f32, q: f32, sample_rate: f32) -> (f32, f32, f32, f32, f32) {
    use std::f32::consts::PI;
    let omega = 2.0 * PI * center_hz / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    let b0 = 1.0;
    let b1 = -2.0 * cos_omega;
    let b2 = 1.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}

// Minimal WAV writer
fn write_wav(path: &str, sample_rate: u32, samples: &[f32]) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::{BufWriter, Write};

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let num_samples = samples.len() as u32;
    let bytes_per_sample = 2u16;
    let num_channels = 1u16;
    let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
    let block_align = num_channels * bytes_per_sample;
    let data_size = num_samples * bytes_per_sample as u32;
    let file_size = 36 + data_size;

    writer.write_all(b"RIFF")?;
    writer.write_all(&file_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&num_channels.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&block_align.to_le_bytes())?;
    writer.write_all(&(bytes_per_sample * 8).to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * 32767.0) as i16;
        writer.write_all(&int_sample.to_le_bytes())?;
    }

    Ok(())
}

// <FILE>examples/test_rust_synth.rs</FILE> - <DESC>KITT scanner sound synthesis test</DESC>
// <VERS>END OF VERSION: 2.1.0</VERS>
