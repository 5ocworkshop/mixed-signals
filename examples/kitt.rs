// <FILE>examples/kitt.rs</FILE> - <DESC>KITT scanner with police lights and siren audio</DESC>
// <VERS>VERSION: 4.2.0 - 2026-01-02</VERS>
// <WCTX>Merge kitt_circular effects and add police siren audio</WCTX>
// <CLOG>Space=mode switch, unique sounds per mode tied to light waveforms</CLOG>

//! # K.I.T.T. Scanner with Police Lights & Siren Audio
//!
//! Demonstrates multiple light effects with synchronized audio:
//!
//! **Light Modes:**
//! - **KITT**: Classic Larson scanner with heartbeat timing
//! - **Band**: Centered band with 3D cylinder rotation
//! - **Full**: Full-width cylinder effect
//! - **Pulse**: Fixed center band that pulses (no lateral movement)
//! - **US Police**: Red/blue alternating wig-wag
//! - **EU Rotate**: European blue rotating beacon
//! - **EU Flash**: European blue rapid strobe
//!
//! **Audio:**
//! - KITT mode: Classic FM-synthesized scanner sweep sound
//! - US Police: Alternating wail/yelp siren (650-1500Hz sweeps)
//! - EU modes: Two-tone "nee-naw" (525Hz/660Hz alternating)
//! - Cylinder modes: Low rumbling pulse with rotation sweep
//!
//! Run with:
//!   Visual only: `cargo run --example kitt --features visualization`
//!   With audio:  `cargo run --example kitt --features "visualization,realtime-audio"`

#[cfg(not(feature = "visualization"))]
fn main() {
    println!("This example requires the 'visualization' feature.");
    println!("Try: cargo run --example kitt --features visualization");
}

#[cfg(feature = "visualization")]
mod tui_demo {
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use mixed_signals::prelude::*;
    use mixed_signals::types::SignalSpec;
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
        Terminal,
    };
    #[cfg(feature = "realtime-audio")]
    use rodio::{OutputStream, Source};
    use std::f64::consts::PI;
    use std::fs;
    use std::io;
    use std::io::Write;
    #[cfg(feature = "realtime-audio")]
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
    #[cfg(feature = "realtime-audio")]
    use std::sync::Arc;

    // Base sound duration for period-adaptive stretching (KITT mode)
    #[cfg(feature = "realtime-audio")]
    const BASE_SOUND_DURATION: f64 = 1.176;

    const CODE_SNIPPET: &str = r#"
// === LIGHT EFFECTS ===
// KITT: Larson Scanner (Triangle Wave LFO)
let position = triangle_wave(time, period);  // 0→1→0
let brightness = (1.0 - distance / falloff).max(0.0);

// Cylinder: 3D rotation with foreshortening
let phi = rotation_angle();  // 0 to 2π
let facing = phi.cos().max(0.0);  // Visible when front-facing

// US Police: Red/blue wig-wag with strobe
let strobe = (time * 12.0 * TAU).sin() > -0.3;
let phase = (time % 0.66) / 0.33;  // Alternate sides

// EU Flash: Triple-flash pattern
let flash_times = [0.0, 0.12, 0.24, 0.5, 0.62, 0.74];

// === AUDIO SYNTHESIS ===
// KITT: FM Synthesis with envelope
let fm_mod = Sine::new(2.19, 1.0, 0.0, 0.0);
let freq = Remap::new(fm_mod, 0.0, 1.0, 380.0, 580.0);
let carrier = PhaseSine::new(PhaseAccumulator::new(freq, 0.0));

// US Wail: Frequency sweep 650Hz → 1500Hz → 650Hz
let wail_mod = Triangle::new(1.0/3.6, 1.0, 0.0, 0.0);
let wail_freq = Remap::new(wail_mod, -1.0, 1.0, 650.0, 1500.0);

// US Yelp: Rapid 8Hz oscillation
let yelp_mod = Sine::new(8.0, 1.0, 0.0, 0.0);
let yelp_freq = Remap::new(yelp_mod, -1.0, 1.0, 650.0, 1500.0);

// EU Two-Tone: 525Hz / 660Hz alternating (nee-naw)
let selector = Square::new(1.0, 1.0, 0.0, 0.0, 0.5);  // 50% duty
let eu_freq = Remap::new(selector, -1.0, 1.0, 525.0, 660.0);
"#;

    // --- Heartbeat-synchronized timing (72 BPM = 0.833s per beat) ---
    const HEARTBEAT_PERIOD: f64 = 5.0 / 6.0;
    const PERIOD_ALERT: f64 = HEARTBEAT_PERIOD * 2.0; // 1.667s
    const PERIOD_CRUISING: f64 = HEARTBEAT_PERIOD * 3.0; // 2.5s
    const PERIOD_MENACING: f64 = HEARTBEAT_PERIOD * 4.0; // 3.333s

    // --- Display modes ---
    #[derive(Clone, Copy, PartialEq, Debug)]
    #[repr(u8)]
    enum DisplayMode {
        Kitt = 0,     // Classic Larson scanner
        CenteredBand, // Cylinder in center band
        FullWidth,    // Cylinder across screen
        FixedPulse,   // Pulsing center band
        UsPolice,     // Red/blue wig-wag
        EuRotate,     // European rotating beacon
        EuFlash,      // European rapid strobe
    }

    impl DisplayMode {
        fn next(self) -> Self {
            match self {
                DisplayMode::Kitt => DisplayMode::CenteredBand,
                DisplayMode::CenteredBand => DisplayMode::FullWidth,
                DisplayMode::FullWidth => DisplayMode::FixedPulse,
                DisplayMode::FixedPulse => DisplayMode::UsPolice,
                DisplayMode::UsPolice => DisplayMode::EuRotate,
                DisplayMode::EuRotate => DisplayMode::EuFlash,
                DisplayMode::EuFlash => DisplayMode::Kitt,
            }
        }

        fn name(&self) -> &'static str {
            match self {
                DisplayMode::Kitt => "KITT",
                DisplayMode::CenteredBand => "Band",
                DisplayMode::FullWidth => "Full",
                DisplayMode::FixedPulse => "Pulse",
                DisplayMode::UsPolice => "US Police",
                DisplayMode::EuRotate => "EU Rotate",
                DisplayMode::EuFlash => "EU Flash",
            }
        }

        fn description(&self) -> &'static str {
            match self {
                DisplayMode::Kitt => "Classic Larson scanner with heartbeat-synced timing",
                DisplayMode::CenteredBand => "Centered band with 3D cylinder rotation inside",
                DisplayMode::FullWidth => "Full-width 3D cylinder rotation effect",
                DisplayMode::FixedPulse => "Fixed band that pulses in/out (no lateral movement)",
                DisplayMode::UsPolice => {
                    "US police wig-wag: red/blue alternating sides with wail siren"
                }
                DisplayMode::EuRotate => "European rotating beacon with two-tone siren",
                DisplayMode::EuFlash => "European rapid triple-flash strobe with two-tone siren",
            }
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum SpeedMode {
        Alert,    // 2 heartbeats - urgent
        Cruising, // 3 heartbeats - relaxed
        Menacing, // 4 heartbeats - slow prowl
    }

    impl SpeedMode {
        fn next(self) -> Self {
            match self {
                SpeedMode::Alert => SpeedMode::Cruising,
                SpeedMode::Cruising => SpeedMode::Menacing,
                SpeedMode::Menacing => SpeedMode::Alert,
            }
        }

        fn period(&self) -> f64 {
            match self {
                SpeedMode::Alert => PERIOD_ALERT,
                SpeedMode::Cruising => PERIOD_CRUISING,
                SpeedMode::Menacing => PERIOD_MENACING,
            }
        }

        fn name(&self) -> &'static str {
            match self {
                SpeedMode::Alert => "Alert (2\u{2665})",
                SpeedMode::Cruising => "Cruising (3\u{2665})",
                SpeedMode::Menacing => "Menacing (4\u{2665})",
            }
        }
    }

    #[derive(Clone)]
    struct ScannerState {
        time: f64,
        period: f64,
        falloff_width: f64,   // For KITT: gradient width as fraction
        band_width: f64,      // For cylinder modes: band width fraction
        angular_falloff: f64, // For cylinder: radians
        speed_mode: SpeedMode,
        display_mode: DisplayMode,
        direction: i8, // 1 = forward/CW, -1 = reverse/CCW
    }

    impl ScannerState {
        fn new() -> Self {
            Self {
                time: 0.0,
                period: PERIOD_ALERT,
                falloff_width: 0.20,
                band_width: 0.25,
                angular_falloff: PI / 4.0,
                speed_mode: SpeedMode::Alert,
                display_mode: DisplayMode::Kitt,
                direction: 1,
            }
        }

        fn toggle_speed(&mut self) {
            self.speed_mode = self.speed_mode.next();
            self.period = self.speed_mode.period();
        }

        fn toggle_mode(&mut self) {
            self.display_mode = self.display_mode.next();
        }

        fn toggle_direction(&mut self) {
            self.direction *= -1;
        }

        fn update(&mut self, dt: f64) {
            self.time += dt;
        }

        // --- KITT Scanner (Triangle wave) ---
        fn triangle_wave(&self) -> f64 {
            let t_norm = (self.time % self.period) / self.period;
            if t_norm < 0.5 {
                t_norm * 2.0
            } else {
                2.0 - (t_norm * 2.0)
            }
        }

        fn kitt_brightness_at(&self, x_norm: f64) -> f64 {
            let center = self.triangle_wave();
            let distance = (center - x_norm).abs();
            (1.0 - (distance / self.falloff_width)).max(0.0)
        }

        // --- Cylinder rotation ---
        fn rotation_angle(&self) -> f64 {
            let t_norm = (self.time % self.period) / self.period;
            let angle = t_norm * 2.0 * PI;
            if self.direction > 0 {
                angle
            } else {
                2.0 * PI - angle
            }
        }

        fn facing_brightness(&self) -> f64 {
            self.rotation_angle().cos().max(0.0)
        }

        fn brightness_full_width(&self, x_norm: f64) -> f64 {
            let phi = self.rotation_angle();
            let theta = (x_norm - 0.5) * PI;
            let angular_dist = (phi.sin() - theta.sin()).abs();
            let proximity = (1.0 - angular_dist / self.angular_falloff.sin()).max(0.0);
            let foreshortening = theta.cos().max(0.0);
            proximity * foreshortening * self.facing_brightness()
        }

        fn brightness_centered_band(&self, x_norm: f64) -> f64 {
            let band_left = 0.5 - self.band_width / 2.0;
            let band_right = 0.5 + self.band_width / 2.0;
            if x_norm < band_left || x_norm > band_right {
                return 0.0;
            }
            let band_pos = (x_norm - band_left) / self.band_width;
            let theta = (band_pos - 0.5) * PI;
            let phi = self.rotation_angle();
            let angular_dist = (phi.sin() - theta.sin()).abs();
            let proximity = (1.0 - angular_dist / self.angular_falloff.sin()).max(0.0);
            let foreshortening = theta.cos().max(0.0);
            proximity * foreshortening * self.facing_brightness()
        }

        fn brightness_fixed_pulse(&self, x_norm: f64) -> f64 {
            let band_left = 0.5 - self.band_width / 2.0;
            let band_right = 0.5 + self.band_width / 2.0;
            if x_norm < band_left || x_norm > band_right {
                return 0.0;
            }
            let band_pos = (x_norm - band_left) / self.band_width;
            let dist_from_center = (band_pos - 0.5).abs() * 2.0;
            let spatial_falloff = (1.0 - dist_from_center * dist_from_center).max(0.0);
            spatial_falloff * self.facing_brightness()
        }

        // --- Police lights ---
        fn color_us_police(&self, x_norm: f64) -> (u8, u8, u8) {
            let flash_period = 0.33;
            let t = self.time % (flash_period * 2.0);
            let phase = t / flash_period;
            let is_left = x_norm < 0.5;
            let strobe_freq = 12.0;
            let strobe = ((self.time * strobe_freq * 2.0 * PI).sin() > -0.3) as u8 as f64;
            let brightness = if phase < 1.0 {
                if is_left {
                    strobe
                } else {
                    0.0
                }
            } else if is_left {
                0.0
            } else {
                strobe
            };
            let half_center = if is_left { 0.25 } else { 0.75 };
            let dist = (x_norm - half_center).abs() / 0.25;
            let falloff = (1.0 - dist * 0.5).max(0.0);
            let final_bright = brightness * falloff;
            let val = (final_bright * 255.0) as u8;
            if is_left {
                (val, 0, 0)
            } else {
                (0, 0, val)
            }
        }

        fn color_eu_rotate(&self, x_norm: f64) -> (u8, u8, u8) {
            let rotation_speed = 1.5;
            let angle = (self.time / self.period) * rotation_speed * 2.0 * PI;
            let theta = (x_norm - 0.5) * PI;
            let lens_angle = if self.direction > 0 { angle } else { -angle };
            let lens_x = lens_angle.sin();
            let screen_x = theta.sin();
            let beam_width = 0.3;
            let dist = (lens_x - screen_x).abs();
            let beam = (1.0 - dist / beam_width).max(0.0);
            let facing = lens_angle.cos().max(0.0);
            let brightness = beam * facing;
            let b = (brightness * 235.0 + 20.0) as u8;
            (20, 20, b)
        }

        fn color_eu_flash(&self, x_norm: f64) -> (u8, u8, u8) {
            let cycle_period = 1.0;
            let t = self.time % cycle_period;
            let flash_duration = 0.05;
            let flash_times = [0.0, 0.12, 0.24, 0.5, 0.62, 0.74];
            let mut is_flash = false;
            for &ft in &flash_times {
                if t >= ft && t < ft + flash_duration {
                    is_flash = true;
                    break;
                }
            }
            let dist_from_center = (x_norm - 0.5).abs() * 2.0;
            let spatial = (1.0 - dist_from_center * 0.3).max(0.0);
            let brightness = if is_flash { spatial } else { 0.0 };
            let b = (brightness * 255.0) as u8;
            (20.min(b), 20.min(b), b)
        }

        /// Returns (red, green, blue) for current mode at position
        fn color_at(&self, x_norm: f64) -> (u8, u8, u8) {
            match self.display_mode {
                DisplayMode::Kitt => {
                    let b = self.kitt_brightness_at(x_norm);
                    let val = (b * 235.0 + 20.0) as u8;
                    (val, 0, 0)
                }
                DisplayMode::CenteredBand => {
                    let b = self.brightness_centered_band(x_norm);
                    let val = (b * 235.0 + 20.0) as u8;
                    (val, 0, 0)
                }
                DisplayMode::FullWidth => {
                    let b = self.brightness_full_width(x_norm);
                    let val = (b * 235.0 + 20.0) as u8;
                    (val, 0, 0)
                }
                DisplayMode::FixedPulse => {
                    let b = self.brightness_fixed_pulse(x_norm);
                    let val = (b * 235.0 + 20.0) as u8;
                    (val, 0, 0)
                }
                DisplayMode::UsPolice => self.color_us_police(x_norm),
                DisplayMode::EuRotate => self.color_eu_rotate(x_norm),
                DisplayMode::EuFlash => self.color_eu_flash(x_norm),
            }
        }
    }

    // --- Describe a SignalSpec for debug output ---
    fn describe_spec(spec: &SignalSpec) -> String {
        match spec {
            SignalSpec::Remap {
                signal,
                in_min,
                in_max,
                out_min,
                out_max,
            } => {
                format!(
                    "Remap[{}→{} to {}→{}]({})",
                    in_min,
                    in_max,
                    out_min,
                    out_max,
                    describe_spec(signal)
                )
            }
            SignalSpec::Sine {
                frequency,
                amplitude,
                ..
            } => {
                format!("Sine({}Hz, amp={})", frequency, amplitude)
            }
            SignalSpec::FrequencyMod {
                carrier_freq,
                depth,
                ..
            } => {
                format!("FM(carrier={}Hz, depth={})", carrier_freq, depth)
            }
            SignalSpec::Add { a, b } => {
                format!("Add({}, {})", describe_spec(a), describe_spec(b))
            }
            SignalSpec::Multiply { a, b } => {
                format!("Mul({}, {})", describe_spec(a), describe_spec(b))
            }
            _ => format!("{:?}", std::mem::discriminant(spec)),
        }
    }

    // =========================================================================
    // Real-time audio (optional feature)
    // =========================================================================
    #[cfg(feature = "realtime-audio")]
    struct SharedAudioState {
        period_us: AtomicU64,
        muted: AtomicBool,
        running: AtomicBool,
        display_mode: AtomicU8,
    }

    #[cfg(feature = "realtime-audio")]
    impl SharedAudioState {
        fn new(initial_period: f64, initial_mode: DisplayMode) -> Self {
            Self {
                period_us: AtomicU64::new((initial_period * 1_000_000.0) as u64),
                muted: AtomicBool::new(false),
                running: AtomicBool::new(true),
                display_mode: AtomicU8::new(initial_mode as u8),
            }
        }
        fn set_period(&self, period: f64) {
            self.period_us
                .store((period * 1_000_000.0) as u64, Ordering::Relaxed);
        }
        fn get_period(&self) -> f64 {
            self.period_us.load(Ordering::Relaxed) as f64 / 1_000_000.0
        }
        fn is_muted(&self) -> bool {
            self.muted.load(Ordering::Relaxed)
        }
        fn toggle_mute(&self) {
            let current = self.muted.load(Ordering::Relaxed);
            self.muted.store(!current, Ordering::Relaxed);
        }
        fn is_running(&self) -> bool {
            self.running.load(Ordering::Relaxed)
        }
        fn stop(&self) {
            self.running.store(false, Ordering::Relaxed);
        }
        fn set_mode(&self, mode: DisplayMode) {
            self.display_mode.store(mode as u8, Ordering::Relaxed);
        }
        fn get_mode(&self) -> DisplayMode {
            match self.display_mode.load(Ordering::Relaxed) {
                0 => DisplayMode::Kitt,
                1 => DisplayMode::CenteredBand,
                2 => DisplayMode::FullWidth,
                3 => DisplayMode::FixedPulse,
                4 => DisplayMode::UsPolice,
                5 => DisplayMode::EuRotate,
                6 => DisplayMode::EuFlash,
                _ => DisplayMode::Kitt,
            }
        }
    }

    #[cfg(feature = "realtime-audio")]
    struct MultiModeAudioSource {
        sample_rate: u32,
        sample_index: u64,
        state: Arc<SharedAudioState>,
        // Filter states
        svf_low: f32,
        svf_band: f32,
        lp_x1: f32,
        lp_x2: f32,
        lp_y1: f32,
        lp_y2: f32,
        nt_x1: f32,
        nt_x2: f32,
        nt_y1: f32,
        nt_y2: f32,
        // KITT oscillator phases
        phase_beat1: f32,
        phase_beat2: f32,
        phase_sub: f32,
        // Siren phases
        siren_phase: f32,
        // RNG
        rng_state: u64,
        // Filter coefficients
        lp_b0: f32,
        lp_b1: f32,
        lp_b2: f32,
        lp_a1: f32,
        lp_a2: f32,
        nt_b0: f32,
        nt_b1: f32,
        nt_b2: f32,
        nt_a1: f32,
        nt_a2: f32,
    }

    #[cfg(feature = "realtime-audio")]
    impl MultiModeAudioSource {
        fn new(sample_rate: u32, state: Arc<SharedAudioState>) -> Self {
            let sr = sample_rate as f32;
            let (lp_b0, lp_b1, lp_b2, lp_a1, lp_a2) =
                Self::biquad_lp(480.0, std::f32::consts::FRAC_1_SQRT_2, sr);
            let (nt_b0, nt_b1, nt_b2, nt_a1, nt_a2) = Self::biquad_notch(450.0, 2.0, sr);
            Self {
                sample_rate,
                sample_index: 0,
                state,
                svf_low: 0.0,
                svf_band: 0.0,
                lp_x1: 0.0,
                lp_x2: 0.0,
                lp_y1: 0.0,
                lp_y2: 0.0,
                nt_x1: 0.0,
                nt_x2: 0.0,
                nt_y1: 0.0,
                nt_y2: 0.0,
                phase_beat1: 0.0,
                phase_beat2: 0.0,
                phase_sub: 0.0,
                siren_phase: 0.0,
                rng_state: 42,
                lp_b0,
                lp_b1,
                lp_b2,
                lp_a1,
                lp_a2,
                nt_b0,
                nt_b1,
                nt_b2,
                nt_a1,
                nt_a2,
            }
        }

        fn biquad_lp(fc: f32, q: f32, sr: f32) -> (f32, f32, f32, f32, f32) {
            use std::f32::consts::PI;
            let w = 2.0 * PI * fc / sr;
            let (c, a) = (w.cos(), w.sin() / (2.0 * q));
            let b1 = 1.0 - c;
            let b0 = b1 / 2.0;
            let a0 = 1.0 + a;
            (b0 / a0, b1 / a0, b0 / a0, -2.0 * c / a0, (1.0 - a) / a0)
        }

        fn biquad_notch(fc: f32, q: f32, sr: f32) -> (f32, f32, f32, f32, f32) {
            use std::f32::consts::PI;
            let w = 2.0 * PI * fc / sr;
            let (c, a) = (w.cos(), w.sin() / (2.0 * q));
            let a0 = 1.0 + a;
            (
                1.0 / a0,
                -2.0 * c / a0,
                1.0 / a0,
                -2.0 * c / a0,
                (1.0 - a) / a0,
            )
        }

        fn next_noise(&mut self) -> f32 {
            self.rng_state = self.rng_state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = self.rng_state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z = z ^ (z >> 31);
            (z as f32 / u64::MAX as f32) * 2.0 - 1.0
        }

        fn kitt_envelope(t: f32) -> f32 {
            const KF: &[(f32, f32)] = &[
                (0.00, 0.20),
                (0.06, 0.22),
                (0.10, 0.45),
                (0.15, 0.90),
                (0.17, 0.70),
                (0.20, 0.55),
                (0.25, 0.62),
                (0.30, 0.60),
                (0.35, 0.58),
                (0.40, 0.60),
                (0.45, 0.72),
                (0.50, 1.00),
                (0.53, 0.80),
                (0.56, 0.55),
                (0.60, 0.35),
                (0.65, 0.18),
                (0.70, 0.08),
                (0.75, 0.03),
                (0.85, 0.01),
                (1.00, 0.00),
                (1.20, 0.00),
            ];
            if t <= KF[0].0 {
                return KF[0].1;
            }
            if t >= KF[KF.len() - 1].0 {
                return KF[KF.len() - 1].1;
            }
            for i in 0..KF.len() - 1 {
                if t >= KF[i].0 && t < KF[i + 1].0 {
                    let a = (t - KF[i].0) / (KF[i + 1].0 - KF[i].0);
                    return KF[i].1 + a * (KF[i + 1].1 - KF[i].1);
                }
            }
            KF[KF.len() - 1].1
        }

        /// Generate KITT scanner sound
        fn generate_kitt(&mut self) -> f32 {
            use std::f32::consts::{PI, TAU};
            let period = self.state.get_period() as f32;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let global_t = self.sample_index as f32 / sr;
            let t_in_period = global_t % period;
            let stretch = period / BASE_SOUND_DURATION as f32;
            let t_sound = t_in_period / stretch;
            let env_val = Self::kitt_envelope(t_sound);
            let base_freq = 380.0 + env_val * 140.0;
            let vib = 3.25 * (TAU * 8.33 * t_sound).sin();
            let freq = base_freq + vib;
            let (f1, f2, fs) = (freq + 13.0, freq - 13.0, freq * 0.5);
            self.phase_beat1 = (self.phase_beat1 + f1 * dt).fract();
            self.phase_beat2 = (self.phase_beat2 + f2 * dt).fract();
            self.phase_sub = (self.phase_sub + fs * dt).fract();
            let raw = ((TAU * self.phase_beat1).sin()
                + (TAU * self.phase_beat2).sin()
                + (TAU * self.phase_sub).sin() * 0.4)
                / 2.4;
            let lp = self.lp_b0 * raw + self.lp_b1 * self.lp_x1 + self.lp_b2 * self.lp_x2
                - self.lp_a1 * self.lp_y1
                - self.lp_a2 * self.lp_y2;
            self.lp_x2 = self.lp_x1;
            self.lp_x1 = raw;
            self.lp_y2 = self.lp_y1;
            self.lp_y1 = lp;
            let nt = self.nt_b0 * lp + self.nt_b1 * self.nt_x1 + self.nt_b2 * self.nt_x2
                - self.nt_a1 * self.nt_y1
                - self.nt_a2 * self.nt_y2;
            self.nt_x2 = self.nt_x1;
            self.nt_x1 = lp;
            self.nt_y2 = self.nt_y1;
            self.nt_y1 = nt;
            let clip = nt.clamp(-1.0, 0.7);
            let white = self.next_noise();
            let fc = base_freq.clamp(20.0, sr * 0.49);
            let f = 2.0 * (PI * fc / sr).sin();
            self.svf_low += f * self.svf_band;
            let hi = white - self.svf_low - self.svf_band / 15.0;
            self.svf_band += f * hi;
            let mix = clip * 0.9 + self.svf_band * 0.1;
            mix * env_val * 0.7
        }

        /// US Police Wail Siren: sweeps 650Hz → 1500Hz → 650Hz over ~1.8s
        fn generate_us_wail(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let global_t = self.sample_index as f32 / sr;

            // Wail cycle: 1.8 seconds up, 1.8 seconds down
            let wail_period = 3.6;
            let t_in_cycle = global_t % wail_period;
            let t_norm = t_in_cycle / wail_period;

            // Triangle wave for frequency modulation
            let freq_mod = if t_norm < 0.5 {
                t_norm * 2.0 // 0 → 1
            } else {
                2.0 - t_norm * 2.0 // 1 → 0
            };

            // Frequency range: 650Hz to 1500Hz
            let freq = 650.0 + freq_mod * 850.0;

            // Generate sine wave at current frequency
            self.siren_phase = (self.siren_phase + freq * dt).fract();
            let sample = (TAU * self.siren_phase).sin();

            // Add slight harmonic content for authenticity
            let harmonic = (TAU * self.siren_phase * 2.0).sin() * 0.15;

            (sample + harmonic) * 0.65
        }

        /// US Police Yelp: rapid frequency oscillation
        fn generate_us_yelp(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let global_t = self.sample_index as f32 / sr;

            // Yelp: very fast oscillation (~8Hz) between frequencies
            let yelp_rate = 8.0;
            let yelp_mod = ((TAU * yelp_rate * global_t).sin() + 1.0) / 2.0;

            // Frequency range: 650Hz to 1500Hz (same as wail but fast)
            let freq = 650.0 + yelp_mod * 850.0;

            self.siren_phase = (self.siren_phase + freq * dt).fract();
            let sample = (TAU * self.siren_phase).sin();

            sample * 0.6
        }

        /// European Two-Tone Siren: alternates 525Hz and 660Hz
        fn generate_eu_two_tone(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let global_t = self.sample_index as f32 / sr;

            // Alternate every 0.5 seconds (classic nee-naw timing)
            let tone_period = 1.0; // Full cycle
            let t_in_cycle = global_t % tone_period;

            // Select frequency: low tone first half, high tone second half
            let freq = if t_in_cycle < 0.5 { 525.0 } else { 660.0 };

            // Smooth transition between tones (avoid clicks)
            let transition_time = 0.02;
            let envelope = if t_in_cycle < transition_time {
                t_in_cycle / transition_time
            } else if t_in_cycle > 0.5 - transition_time && t_in_cycle < 0.5 + transition_time {
                let dist = (t_in_cycle - 0.5).abs();
                dist / transition_time
            } else if t_in_cycle > 1.0 - transition_time {
                (1.0 - t_in_cycle) / transition_time
            } else {
                1.0
            };

            self.siren_phase = (self.siren_phase + freq * dt).fract();
            let sample = (TAU * self.siren_phase).sin();

            sample * envelope * 0.6
        }

        /// CenteredBand: Contained turbine - resonant drone with metallic harmonics
        fn generate_band_turbine(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let period = self.state.get_period() as f32;
            let global_t = self.sample_index as f32 / sr;
            let t_in_period = global_t % period;
            let t_norm = t_in_period / period;

            // Rotation creates Doppler-like pitch shift
            let angle = t_norm * TAU;
            let facing = angle.cos().max(0.0);
            let doppler = 1.0 + angle.sin() * 0.08; // Slight pitch wobble

            // Turbine base frequency with harmonics (like contained machinery)
            let base_freq = 120.0 * doppler;
            self.siren_phase = (self.siren_phase + base_freq * dt).fract();

            // Metallic harmonics (2nd, 3rd, 5th)
            let h1 = (TAU * self.siren_phase).sin();
            let h2 = (TAU * self.siren_phase * 2.0).sin() * 0.4;
            let h3 = (TAU * self.siren_phase * 3.0).sin() * 0.2;
            let h5 = (TAU * self.siren_phase * 5.0).sin() * 0.1;

            // Resonant chamber effect (filtered noise)
            let chamber_noise = self.next_noise() * 0.08 * facing;

            let mix = (h1 + h2 + h3 + h5) / 1.7 + chamber_noise;
            mix * facing * 0.5
        }

        /// FullWidth: Sweeping scanner - wide frequency sweep, sci-fi whoosh
        fn generate_full_sweep(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let period = self.state.get_period() as f32;
            let global_t = self.sample_index as f32 / sr;
            let t_in_period = global_t % period;
            let t_norm = t_in_period / period;

            // Full sweep across frequency range as it rotates
            let angle = t_norm * TAU;
            let facing = angle.cos().max(0.0);

            // Frequency sweeps from 200Hz to 800Hz based on position
            let sweep_freq = 200.0 + (angle.sin() + 1.0) * 300.0;
            self.siren_phase = (self.siren_phase + sweep_freq * dt).fract();

            // Sub-bass rumble
            self.phase_sub = (self.phase_sub + 50.0 * dt).fract();
            let sub = (TAU * self.phase_sub).sin() * 0.3;

            // Main tone with slight FM for "electronic" quality
            let fm = (global_t * 6.0 * TAU).sin() * 0.02;
            let main = (TAU * (self.siren_phase + fm)).sin();

            // Whoosh noise synchronized with sweep
            let whoosh = self.next_noise() * 0.2 * facing * (1.0 - facing);

            (main * 0.6 + sub + whoosh) * (facing * 0.7 + 0.1)
        }

        /// FixedPulse: Heartbeat - deep rhythmic thump synchronized with visual pulse
        fn generate_pulse_heartbeat(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let sr = self.sample_rate as f32;
            let dt = 1.0 / sr;
            let period = self.state.get_period() as f32;
            let global_t = self.sample_index as f32 / sr;
            let t_in_period = global_t % period;
            let t_norm = t_in_period / period;

            // Pulse envelope (sharp attack, gradual decay) - like a heartbeat
            let angle = t_norm * TAU;
            let facing = angle.cos().max(0.0);

            // Double-beat pattern (lub-dub) within each rotation
            let beat_phase = t_norm * 2.0; // Two beats per rotation
            let beat1 = if beat_phase < 0.15 {
                (beat_phase / 0.15).powf(0.3) * (1.0 - beat_phase / 0.15).powf(2.0) * 4.0
            } else {
                0.0
            };
            let beat2 = if beat_phase > 0.2 && beat_phase < 0.35 {
                let bp = (beat_phase - 0.2) / 0.15;
                bp.powf(0.3) * (1.0 - bp).powf(2.0) * 2.5
            } else {
                0.0
            };
            let beat_env = (beat1 + beat2).min(1.0);

            // Deep bass thump
            let thump_freq = 45.0 + beat_env * 15.0;
            self.siren_phase = (self.siren_phase + thump_freq * dt).fract();
            let thump = (TAU * self.siren_phase).sin();

            // Body resonance
            self.phase_sub = (self.phase_sub + 90.0 * dt).fract();
            let body = (TAU * self.phase_sub).sin() * 0.3 * beat_env;

            // Subtle high transient on beat
            let click = self.next_noise() * beat_env * 0.15;

            (thump + body + click) * beat_env.max(0.05) * facing.max(0.3) * 0.6
        }

        fn generate_sample(&mut self) -> f32 {
            let mode = self.state.get_mode();
            let sample = match mode {
                DisplayMode::Kitt => self.generate_kitt(),
                DisplayMode::CenteredBand => self.generate_band_turbine(),
                DisplayMode::FullWidth => self.generate_full_sweep(),
                DisplayMode::FixedPulse => self.generate_pulse_heartbeat(),
                DisplayMode::UsPolice => {
                    // Alternate between wail and yelp every 4 seconds
                    let global_t = self.sample_index as f32 / self.sample_rate as f32;
                    if (global_t as i32 / 4) % 2 == 0 {
                        self.generate_us_wail()
                    } else {
                        self.generate_us_yelp()
                    }
                }
                DisplayMode::EuRotate | DisplayMode::EuFlash => self.generate_eu_two_tone(),
            };
            self.sample_index += 1;
            sample
        }
    }

    #[cfg(feature = "realtime-audio")]
    impl Iterator for MultiModeAudioSource {
        type Item = f32;
        fn next(&mut self) -> Option<f32> {
            if !self.state.is_running() {
                return None;
            }
            if self.state.is_muted() {
                self.sample_index += 1;
                Some(0.0)
            } else {
                Some(self.generate_sample())
            }
        }
    }

    #[cfg(feature = "realtime-audio")]
    impl Source for MultiModeAudioSource {
        fn current_frame_len(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> u16 {
            1
        }
        fn sample_rate(&self) -> u32 {
            self.sample_rate
        }
        fn total_duration(&self) -> Option<std::time::Duration> {
            None
        }
    }

    // --- Audio Generation (WAV export) ---
    fn save_wav(
        mode: DisplayMode,
        custom_spec: Option<SignalSpec>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use mixed_signals::generators::{PhaseAccumulator, PhaseSine, Square, Triangle};
        use mixed_signals::traits::SignalExt;
        use std::f64::consts::TAU;

        let sample_rate = 48000_u32;
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        // Duration varies by mode
        let (duration, filename): (f64, &str) = match mode {
            DisplayMode::Kitt => (1.1757, "kitt_scanner.wav"),
            DisplayMode::CenteredBand => (5.0, "band_turbine.wav"),
            DisplayMode::FullWidth => (5.0, "full_sweep.wav"),
            DisplayMode::FixedPulse => (5.0, "pulse_heartbeat.wav"),
            DisplayMode::UsPolice => (8.0, "us_police_siren.wav"),
            DisplayMode::EuRotate => (4.0, "eu_rotate_siren.wav"),
            DisplayMode::EuFlash => (4.0, "eu_flash_siren.wav"),
        };

        let mut writer = hound::WavWriter::create(filename, spec)?;
        let num_samples = (duration * sample_rate as f64) as usize;
        let dt = 1.0 / sample_rate as f64;

        let mut log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("kitt_debug.log")
            .ok();

        if let Some(signal_spec) = custom_spec {
            if let Some(ref mut f) = log {
                let _ = writeln!(f, "=== USING JSON PATH ===");
                let _ = writeln!(f, "Spec description: {}", describe_spec(&signal_spec));
            }
            let signal = signal_spec
                .build()
                .map_err(|e| format!("Signal build error: {}", e))?;
            for i in 0..num_samples {
                let t = i as f64 * dt;
                let sample = signal.sample(t).clamp(-1.0, 1.0);
                writer.write_sample((sample * i16::MAX as f32) as i16)?;
            }
        } else {
            match mode {
                DisplayMode::Kitt => {
                    // KITT: True FM Synthesis (V17 Recipe)
                    let center_freq = 480.0_f32;
                    let freq_deviation = 100.0_f32;
                    let fm_rate_hz = 2.19_f32;
                    let noise_mix = 0.08_f32;
                    let output_scale = 0.80_f32;
                    let silence_start = 0.2_f64;
                    let active_duration = 0.685_f64;
                    let attack = 0.05_f64;
                    let release = 0.12_f64;

                    let fm_modulator = Sine::new(fm_rate_hz, 1.0, 0.0, 0.0);
                    let freq_signal = Remap::new(
                        fm_modulator,
                        0.0,
                        1.0,
                        center_freq - freq_deviation,
                        center_freq + freq_deviation,
                    );
                    let phase_signal = PhaseAccumulator::new(freq_signal, 0.0);
                    let carrier = PhaseSine::new(phase_signal);
                    let noise = Remap::to_bipolar(PinkNoise::new(42, 1.0, 0.0));
                    let mixed = carrier.mix(noise, noise_mix);
                    let tremolo = Sine::new(fm_rate_hz, 0.35, 0.65, 0.0);
                    let enveloped = mixed.multiply(tremolo);
                    let scaled = enveloped.scale(output_scale);

                    for i in 0..num_samples {
                        let t_global = i as f64 * dt;
                        let sample = if t_global < silence_start {
                            0.0_f32
                        } else if t_global < silence_start + active_duration {
                            let t_audio = t_global - silence_start;
                            let t_norm = t_audio / active_duration;
                            let raw = scaled.sample(t_audio);
                            let macro_env = if t_norm < attack {
                                (t_norm / attack) as f32
                            } else if t_norm > 1.0 - release {
                                ((1.0 - t_norm) / release) as f32
                            } else {
                                1.0_f32
                            };
                            (raw * macro_env).clamp(-1.0, 1.0)
                        } else {
                            0.0_f32
                        };
                        writer.write_sample((sample * i16::MAX as f32) as i16)?;
                    }
                }
                DisplayMode::UsPolice => {
                    // US Police: Wail (first 4s) then Yelp (next 4s)
                    // Wail: Triangle wave modulating frequency 650-1500Hz over 3.6s
                    let wail_mod = Triangle::new(1.0 / 3.6, 1.0, 0.0, 0.0);
                    let wail_freq = Remap::new(wail_mod, -1.0, 1.0, 650.0, 1500.0);
                    let wail_phase = PhaseAccumulator::new(wail_freq, 0.0);
                    let wail_carrier = PhaseSine::new(wail_phase);

                    // Yelp: Sine wave at 8Hz modulating frequency
                    let yelp_mod = Sine::new(8.0, 1.0, 0.0, 0.0);
                    let yelp_freq = Remap::new(yelp_mod, -1.0, 1.0, 650.0, 1500.0);
                    let yelp_phase = PhaseAccumulator::new(yelp_freq, 0.0);
                    let yelp_carrier = PhaseSine::new(yelp_phase);

                    for i in 0..num_samples {
                        let t = i as f64 * dt;
                        let sample = if t < 4.0 {
                            // Wail phase
                            wail_carrier.sample(t) * 0.65
                        } else {
                            // Yelp phase (offset time for clean start)
                            yelp_carrier.sample(t - 4.0) * 0.6
                        };
                        writer.write_sample((sample * i16::MAX as f32) as i16)?;
                    }
                }
                DisplayMode::EuRotate | DisplayMode::EuFlash => {
                    // EU Two-Tone: Square wave selecting between 525Hz and 660Hz
                    let selector = Square::new(1.0, 1.0, 0.0, 0.0, 0.5); // 1Hz, 50% duty = 0.5s each tone
                    let eu_freq = Remap::new(selector, -1.0, 1.0, 525.0, 660.0);
                    let eu_phase = PhaseAccumulator::new(eu_freq, 0.0);
                    let eu_carrier = PhaseSine::new(eu_phase);

                    for i in 0..num_samples {
                        let t = i as f64 * dt;
                        let sample = eu_carrier.sample(t) * 0.6;
                        writer.write_sample((sample * i16::MAX as f32) as i16)?;
                    }
                }
                DisplayMode::CenteredBand => {
                    // Band Turbine: Contained machinery with Doppler shift
                    // Uses Triangle for rotation, harmonics for metallic quality
                    let period = 2.5_f64;
                    let rotation = Triangle::new(1.0 / period as f32, 1.0, 0.0, 0.0);
                    // Base frequency modulated by rotation (Doppler effect)
                    let base_freq = Remap::new(rotation, -1.0, 1.0, 110.0, 130.0);
                    let base_phase = PhaseAccumulator::new(base_freq, 0.0);
                    let carrier = PhaseSine::new(base_phase);
                    // Harmonics for metallic resonance
                    let h2_freq = Remap::new(
                        Triangle::new(1.0 / period as f32, 1.0, 0.0, 0.0),
                        -1.0,
                        1.0,
                        220.0,
                        260.0,
                    );
                    let h2 = PhaseSine::new(PhaseAccumulator::new(h2_freq, 0.0));

                    for i in 0..num_samples {
                        let t = i as f64 * dt;
                        let t_norm = (t % period) / period;
                        let angle = t_norm * TAU;
                        let facing = angle.cos().max(0.0) as f32;
                        let main = carrier.sample(t);
                        let harmonic = h2.sample(t) * 0.4;
                        let sample = (main + harmonic) * facing * 0.45;
                        writer.write_sample((sample * i16::MAX as f32) as i16)?;
                    }
                }
                DisplayMode::FullWidth => {
                    // Full Sweep: Sci-fi scanner with wide frequency sweep
                    // Uses Sine for smooth sweep, sub-bass for weight
                    let period = 2.5_f64;
                    let sweep_lfo = Sine::new(1.0 / period as f32, 1.0, 0.0, 0.0);
                    let sweep_freq = Remap::new(sweep_lfo, -1.0, 1.0, 200.0, 800.0);
                    let sweep_phase = PhaseAccumulator::new(sweep_freq, 0.0);
                    let sweep_carrier = PhaseSine::new(sweep_phase);
                    let sub = Sine::new(50.0, 0.3, 0.0, 0.0);

                    for i in 0..num_samples {
                        let t = i as f64 * dt;
                        let t_norm = (t % period) / period;
                        let angle = t_norm * TAU;
                        let facing = (angle.cos().max(0.0) * 0.7 + 0.1) as f32;
                        let main = sweep_carrier.sample(t) * 0.6;
                        let bass = sub.sample(t);
                        let sample = (main + bass) * facing;
                        writer.write_sample((sample * i16::MAX as f32) as i16)?;
                    }
                }
                DisplayMode::FixedPulse => {
                    // Pulse Heartbeat: Deep rhythmic thump (lub-dub pattern)
                    // Uses low frequency with sharp envelope for cardiac rhythm
                    let period = 2.5_f64;
                    let thump = Sine::new(45.0, 1.0, 0.0, 0.0);
                    let body = Sine::new(90.0, 0.3, 0.0, 0.0);

                    for i in 0..num_samples {
                        let t = i as f64 * dt;
                        let t_norm = (t % period) / period;
                        let angle = t_norm * TAU;
                        let facing = (angle.cos().max(0.0) as f32).max(0.3);

                        // Double-beat envelope (lub-dub)
                        let beat_phase = t_norm * 2.0;
                        let beat1 = if beat_phase < 0.15 {
                            ((beat_phase / 0.15).powf(0.3)
                                * (1.0 - beat_phase / 0.15).powf(2.0)
                                * 4.0) as f32
                        } else {
                            0.0
                        };
                        let beat2 = if beat_phase > 0.2 && beat_phase < 0.35 {
                            let bp = (beat_phase - 0.2) / 0.15;
                            (bp.powf(0.3) * (1.0 - bp).powf(2.0) * 2.5) as f32
                        } else {
                            0.0
                        };
                        let beat_env = (beat1 + beat2).clamp(0.05, 1.0);

                        let main = thump.sample(t);
                        let resonance = body.sample(t) * beat_env;
                        let sample = (main + resonance) * beat_env * facing * 0.6;
                        writer.write_sample((sample * i16::MAX as f32) as i16)?;
                    }
                }
            }
        }
        writer.finalize()?;
        Ok(filename.to_string())
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // --- Real-time audio setup ---
        #[cfg(feature = "realtime-audio")]
        let audio_state = Arc::new(SharedAudioState::new(PERIOD_ALERT, DisplayMode::Kitt));
        #[cfg(feature = "realtime-audio")]
        let (_stream, audio_available) = {
            let state = Arc::clone(&audio_state);
            match OutputStream::try_default() {
                Ok((stream, handle)) => {
                    let source = MultiModeAudioSource::new(48000, state);
                    match handle.play_raw(source.convert_samples::<f32>()) {
                        Ok(_) => (Some(stream), true),
                        Err(_) => (None, false),
                    }
                }
                Err(_) => (None, false),
            }
        };
        #[cfg(not(feature = "realtime-audio"))]
        let audio_available = false;

        // --- Signal Setup ---
        let voice_signal = PinkNoise::new(1982, 1.0, 0.0);
        let mut scanner = ScannerState::new();
        let mut time = 0.0;
        let mut show_code = false;
        let mut status_msg: Option<(String, f64)> = None;
        let mut loaded_spec: Option<SignalSpec> = None;

        loop {
            #[cfg(feature = "realtime-audio")]
            let is_muted = audio_available && audio_state.is_muted();
            #[cfg(not(feature = "realtime-audio"))]
            let is_muted = false;

            terminal.draw(|f| {
                let area = f.area();
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(6), // Header
                        Constraint::Length(5), // Scanner
                        Constraint::Length(5), // Voice Box
                        Constraint::Min(0),    // Footer
                    ])
                    .margin(2)
                    .split(area);

                // --- Header ---
                let explanation = vec![
                    Line::from(vec![
                        Span::styled(
                            scanner.display_mode.name(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(": "),
                        Span::raw(scanner.display_mode.description()),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Modes: ", Style::default().fg(Color::Gray)),
                        Span::styled("KITT", Style::default().fg(Color::Red)),
                        Span::raw(" \u{2192} "),
                        Span::styled("Band", Style::default().fg(Color::Red)),
                        Span::raw(" \u{2192} "),
                        Span::styled("Full", Style::default().fg(Color::Red)),
                        Span::raw(" \u{2192} "),
                        Span::styled("Pulse", Style::default().fg(Color::Red)),
                        Span::raw(" \u{2192} "),
                        Span::styled("US Police", Style::default().fg(Color::Magenta)),
                        Span::raw(" \u{2192} "),
                        Span::styled("EU", Style::default().fg(Color::Blue)),
                    ]),
                ];
                let header_block = Paragraph::new(explanation)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Light & Sound Effects ")
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
                    .wrap(Wrap { trim: true });
                f.render_widget(header_block, vertical_chunks[0]);

                // --- Scanner Rendering ---
                let scanner_area = vertical_chunks[1];
                let inner_scanner = Rect::new(
                    scanner_area.x + 2,
                    scanner_area.y + 1,
                    scanner_area.width - 4,
                    3,
                );

                let mut styles = Vec::new();
                for x in 0..inner_scanner.width {
                    let x_norm = x as f64 / (inner_scanner.width - 1).max(1) as f64;
                    let (r, g, b) = scanner.color_at(x_norm);
                    styles.push(Style::default().fg(Color::Rgb(r, g, b)));
                }

                for y in 0..3 {
                    let spans: Vec<_> = styles
                        .iter()
                        .take(inner_scanner.width as usize)
                        .map(|s| Span::styled("\u{2588}", *s))
                        .collect();
                    let p = Paragraph::new(Line::from(spans));
                    f.render_widget(
                        p,
                        Rect::new(inner_scanner.x, inner_scanner.y + y, inner_scanner.width, 1),
                    );
                }

                // Scanner title varies by mode
                let title = match scanner.display_mode {
                    DisplayMode::Kitt => {
                        let t_norm = (scanner.time % scanner.period) / scanner.period;
                        let dir = if t_norm < 0.5 { "\u{25b6}" } else { "\u{25c0}" };
                        format!(" KNIGHT INDUSTRIES TWO THOUSAND {} ", dir)
                    }
                    DisplayMode::UsPolice => " EMERGENCY RESPONSE UNIT ".to_string(),
                    DisplayMode::EuRotate | DisplayMode::EuFlash => {
                        " POLIZEI / POLICE ".to_string()
                    }
                    _ => {
                        let dir = if scanner.direction > 0 { "CW" } else { "CCW" };
                        let deg = (scanner.rotation_angle() * 180.0 / PI) as i32 % 360;
                        format!(" CYLINDER {} | {}\u{00b0} ", dir, deg)
                    }
                };
                let scanner_block = Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::DarkGray));
                f.render_widget(scanner_block, scanner_area);

                // --- Voice Box Rendering ---
                let voice_area = vertical_chunks[2];
                let inner_voice =
                    Rect::new(voice_area.x + 2, voice_area.y + 1, voice_area.width - 4, 3);
                let voice_val = voice_signal.sample(time * 8.0);
                let center_x = inner_voice.width / 2;
                let mut voice_line = String::new();
                let mut voice_styles = Vec::new();
                for x in 0..inner_voice.width {
                    let dist_from_center = (x as i32 - center_x as i32).abs() as f32;
                    let max_dist = center_x as f32;
                    let norm_dist = dist_from_center / max_dist;
                    let is_lit = norm_dist < voice_val;
                    if is_lit {
                        voice_line.push('\u{2502}');
                        let amp = voice_val;
                        let color = if amp < 0.5 {
                            let b = (amp / 0.5 * 235.0 + 20.0) as u8;
                            Color::Rgb(b, 0, 0)
                        } else if amp < 0.75 {
                            let b = ((amp - 0.5) / 0.25 * 235.0 + 20.0) as u8;
                            Color::Rgb(b, b, 0)
                        } else {
                            let b = ((amp - 0.75) / 0.25 * 235.0 + 20.0) as u8;
                            Color::Rgb(20, b, 20)
                        };
                        voice_styles.push(Style::default().fg(color));
                    } else {
                        voice_line.push(' ');
                        voice_styles.push(Style::default());
                    }
                }
                for y in 0..3 {
                    let mut spans = Vec::new();
                    for (i, ch) in voice_line.chars().enumerate() {
                        spans.push(Span::styled(ch.to_string(), voice_styles[i]));
                    }
                    let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
                    f.render_widget(
                        p,
                        Rect::new(inner_voice.x, inner_voice.y + y, inner_voice.width, 1),
                    );
                }
                let voice_block = Block::default()
                    .borders(Borders::ALL)
                    .title(" VOICE MODULATOR ")
                    .border_style(Style::default().fg(Color::DarkGray));
                f.render_widget(voice_block, voice_area);

                // --- Footer ---
                let audio_mode = if loaded_spec.is_some() {
                    "[JSON]"
                } else {
                    "[Native]"
                };
                let rt_audio_indicator = if !audio_available {
                    ("", Color::DarkGray)
                } else if is_muted {
                    ("[MUTED] ", Color::DarkGray)
                } else {
                    ("[♪] ", Color::Green)
                };
                let mut footer_text = vec![
                    Span::styled(
                        rt_audio_indicator.0,
                        Style::default().fg(rt_audio_indicator.1),
                    ),
                    Span::styled(
                        audio_mode,
                        Style::default().fg(if loaded_spec.is_some() {
                            Color::Green
                        } else {
                            Color::Gray
                        }),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        scanner.display_mode.name(),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::styled(scanner.speed_mode.name(), Style::default().fg(Color::Gray)),
                    Span::raw(" | "),
                    Span::styled("[space]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Next"),
                    Span::raw(" | "),
                    Span::styled("[t]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Speed"),
                    Span::raw(" | "),
                    Span::styled("[d]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Dir"),
                    Span::raw(" | "),
                    Span::styled("[r]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Reset"),
                ];
                if audio_available {
                    footer_text.push(Span::raw(" | "));
                    footer_text.push(Span::styled("[s]", Style::default().fg(Color::Yellow)));
                    footer_text.push(Span::raw(" Mute"));
                }
                footer_text.extend([
                    Span::raw(" | "),
                    Span::styled("[c]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Code | "),
                    Span::styled("[q]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Quit"),
                ]);
                if let Some((msg, timestamp)) = &status_msg {
                    if time - timestamp < 3.0 {
                        footer_text.push(Span::raw(" | "));
                        footer_text.push(Span::styled(
                            msg,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                }
                let footer = Paragraph::new(Line::from(footer_text))
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                f.render_widget(footer, vertical_chunks[3]);

                // --- Code Overlay ---
                if show_code {
                    let popup_area = centered_rect(70, 60, area);
                    f.render_widget(Clear, popup_area);
                    let code_block = Paragraph::new(CODE_SNIPPET)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Audio Synthesis Examples ")
                                .border_style(Style::default().fg(Color::Yellow)),
                        )
                        .style(Style::default().fg(Color::White));
                    f.render_widget(code_block, popup_area);
                }
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            #[cfg(feature = "realtime-audio")]
                            if audio_available {
                                audio_state.stop();
                            }
                            break;
                        }
                        KeyCode::Char(' ') => {
                            scanner.toggle_mode();
                            #[cfg(feature = "realtime-audio")]
                            if audio_available {
                                audio_state.set_mode(scanner.display_mode);
                            }
                            status_msg = Some((scanner.display_mode.name().to_string(), time));
                        }
                        KeyCode::Char('r') => {
                            scanner = ScannerState::new();
                            time = 0.0;
                            loaded_spec = None;
                            #[cfg(feature = "realtime-audio")]
                            if audio_available {
                                audio_state.set_period(scanner.period);
                                audio_state.set_mode(scanner.display_mode);
                            }
                            status_msg = Some(("Reset".to_string(), time));
                        }
                        KeyCode::Char('t') => {
                            scanner.toggle_speed();
                            #[cfg(feature = "realtime-audio")]
                            if audio_available {
                                audio_state.set_period(scanner.period);
                            }
                            status_msg =
                                Some((format!("Speed: {}", scanner.speed_mode.name()), time));
                        }
                        KeyCode::Char('d') => {
                            scanner.toggle_direction();
                            status_msg = Some((
                                format!(
                                    "Direction: {}",
                                    if scanner.direction > 0 {
                                        "Forward"
                                    } else {
                                        "Reverse"
                                    }
                                ),
                                time,
                            ));
                        }
                        KeyCode::Char('s') =>
                        {
                            #[cfg(feature = "realtime-audio")]
                            if audio_available {
                                audio_state.toggle_mute();
                                let msg = if audio_state.is_muted() {
                                    "Audio muted"
                                } else {
                                    "Audio unmuted"
                                };
                                status_msg = Some((msg.to_string(), time));
                            }
                        }
                        KeyCode::Char('c') => show_code = !show_code,
                        KeyCode::Char('w') => {
                            let spec_to_use = loaded_spec.clone();
                            let source = if spec_to_use.is_some() {
                                "JSON"
                            } else {
                                "native"
                            };
                            match save_wav(scanner.display_mode, spec_to_use) {
                                Ok(filename) => {
                                    status_msg = Some((
                                        format!(
                                            "Saved {} ({}, {})",
                                            filename,
                                            scanner.display_mode.name(),
                                            source
                                        ),
                                        time,
                                    ))
                                }
                                Err(e) => status_msg = Some((format!("Error: {}", e), time)),
                            }
                        }
                        KeyCode::Char('f') => {
                            let cwd = std::env::current_dir()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|_| "unknown".to_string());
                            match fs::read_to_string("kitt.json") {
                                Ok(json) => match serde_json::from_str::<SignalSpec>(&json) {
                                    Ok(spec) => {
                                        let spec_desc = describe_spec(&spec);
                                        loaded_spec = Some(spec);
                                        status_msg = Some((
                                            format!("Loaded: {} - press 'w' to save", spec_desc),
                                            time,
                                        ))
                                    }
                                    Err(e) => {
                                        status_msg = Some((format!("JSON Error: {}", e), time))
                                    }
                                },
                                Err(e) => {
                                    status_msg = Some((
                                        format!("kitt.json not found in {}: {}", cwd, e),
                                        time,
                                    ))
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            scanner.update(0.016);
            time += 0.016;
        }
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[cfg(feature = "visualization")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui_demo::run()
}

// <FILE>examples/kitt.rs</FILE> - <DESC>KITT scanner with police lights and siren audio</DESC>
// <VERS>END OF VERSION: 4.2.0 - 2026-01-02</VERS>
