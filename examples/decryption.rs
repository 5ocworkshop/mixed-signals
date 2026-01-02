// <FILE>examples/decryption.rs</FILE> - <DESC>Demonstrates per-character deterministic noise with Ratatui</DESC>
// <VERS>VERSION: 2.0.0 - 2026-01-02</VERS>
// <WCTX>Add audio driven by cycling letters</WCTX>
// <CLOG>Added pentatonic data stream sound tied to character cycling</CLOG>

//! # Decryption Effect Example (Ratatui + Audio)
//!
//! Demonstrates:
//! 1. `PerCharacterNoise`: Generating stable random values per character index.
//! 2. `SignalContext`: Linking noise to specific character positions.
//! 3. `Ratatui`: Rendering the effect in a TUI.
//! 4. **Audio**: Pleasant pentatonic tones driven by cycling characters.
//!
//! **Note:** Ratatui is OPTIONAL. mixed-signals works with any UI or backend.
//!
//! Run with:
//!   Visual only: `cargo run --example decryption --features visualization`
//!   With audio:  `cargo run --example decryption --features "visualization,realtime-audio"`
#[cfg(not(feature = "visualization"))]
fn main() {
    println!("This example requires the 'visualization' feature.");
    println!("Try: cargo run --example decryption --features visualization");
}
#[cfg(feature = "visualization")]
mod tui_demo {
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use mixed_signals::prelude::*;
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
    use std::io;
    #[cfg(feature = "realtime-audio")]
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    #[cfg(feature = "realtime-audio")]
    use std::sync::Arc;

    // Lower pentatonic scale (C3 to G4) - warm, non-irritating range
    // Each cycling character maps directly to a note based on its ASCII value
    #[cfg(feature = "realtime-audio")]
    const PENTATONIC: [f32; 8] = [
        130.81, 146.83, 164.81, 196.00, 220.00, 261.63, 293.66, 392.00,
    ];

    #[cfg(feature = "realtime-audio")]
    struct SharedDecryptState {
        current_char: AtomicU32,  // Current cycling character (ASCII value)
        active_chars: AtomicU32,  // Number of currently cycling characters
        char_changed: AtomicBool, // Flag: character just changed this frame
        muted: AtomicBool,
        running: AtomicBool,
    }

    #[cfg(feature = "realtime-audio")]
    impl SharedDecryptState {
        fn new() -> Self {
            Self {
                current_char: AtomicU32::new(0),
                active_chars: AtomicU32::new(0),
                char_changed: AtomicBool::new(false),
                muted: AtomicBool::new(false),
                running: AtomicBool::new(true),
            }
        }
        fn set_char(&self, c: u32) {
            let old = self.current_char.swap(c, Ordering::Relaxed);
            if old != c && c > 0 {
                self.char_changed.store(true, Ordering::Relaxed);
            }
        }
        fn get_char(&self) -> u32 {
            self.current_char.load(Ordering::Relaxed)
        }
        fn set_active(&self, n: u32) {
            self.active_chars.store(n, Ordering::Relaxed);
        }
        fn get_active(&self) -> u32 {
            self.active_chars.load(Ordering::Relaxed)
        }
        #[allow(dead_code)]
        fn take_changed(&self) -> bool {
            self.char_changed.swap(false, Ordering::Relaxed)
        }
        fn is_muted(&self) -> bool {
            self.muted.load(Ordering::Relaxed)
        }
        fn toggle_mute(&self) {
            let cur = self.muted.load(Ordering::Relaxed);
            self.muted.store(!cur, Ordering::Relaxed);
        }
        fn is_running(&self) -> bool {
            self.running.load(Ordering::Relaxed)
        }
        fn stop(&self) {
            self.running.store(false, Ordering::Relaxed);
        }
    }

    #[cfg(feature = "realtime-audio")]
    struct DecryptAudioSource {
        sample_rate: u32,
        sample_index: u64,
        state: Arc<SharedDecryptState>,
        phase: f32,
        target_freq: f32,
        current_freq: f32,
        envelope: f32,
        pluck_env: f32, // Short pluck envelope for each character change
        last_char: u32,
    }

    #[cfg(feature = "realtime-audio")]
    impl DecryptAudioSource {
        fn new(sample_rate: u32, state: Arc<SharedDecryptState>) -> Self {
            Self {
                sample_rate,
                sample_index: 0,
                state,
                phase: 0.0,
                target_freq: PENTATONIC[0],
                current_freq: PENTATONIC[0],
                envelope: 0.0,
                pluck_env: 0.0,
                last_char: 0,
            }
        }

        fn generate_sample(&mut self) -> f32 {
            use std::f32::consts::TAU;
            let dt = 1.0 / self.sample_rate as f32;

            // Get current character - each character directly determines the note
            let char_val = self.state.get_char();
            let active = self.state.get_active();

            // Detect character changes for pluck trigger
            if char_val != self.last_char && char_val > 0 {
                // New character = new pluck attack
                self.pluck_env = 1.0;
                // Map character directly to note (creates melody from the cycling)
                let note_idx = (char_val as usize) % PENTATONIC.len();
                self.target_freq = PENTATONIC[note_idx];
                self.last_char = char_val;
            }

            // Pluck envelope: fast decay (each character gets a soft "tick")
            self.pluck_env = (self.pluck_env - dt * 4.0).max(0.0);

            if active > 0 {
                // Ambient envelope rises with activity
                let target_env = (active as f32 / 50.0).min(0.25);
                self.envelope = self.envelope + (target_env - self.envelope) * dt * 3.0;
            } else {
                // Fade out when no cycling
                self.envelope = (self.envelope - dt * 1.5).max(0.0);
            }

            // Slow, smooth frequency glide (lazy portamento)
            let freq_diff = self.target_freq - self.current_freq;
            self.current_freq += freq_diff * dt * 6.0;

            // Generate warm sine tone
            self.phase = (self.phase + self.current_freq * dt).fract();
            let main = (TAU * self.phase).sin();

            // Soft sub-octave for warmth
            let sub = (TAU * self.phase * 0.5).sin() * 0.3;

            // Combine: ambient drone + pluck accents tied to character changes
            let ambient = (main + sub) * self.envelope * 0.25;
            let pluck = main * self.pluck_env * 0.4;

            self.sample_index += 1;
            (ambient + pluck).clamp(-0.8, 0.8) * 0.5
        }
    }

    #[cfg(feature = "realtime-audio")]
    impl Iterator for DecryptAudioSource {
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
    impl Source for DecryptAudioSource {
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
    const CODE_SNIPPET: &str = r#"
// 1. Setup Noise
// PerCharacterNoise is stable for a given index
let noise = PerCharacterNoise::new(0xCAFEBABE, 1.0, 0.0);
// 2. In the loop
// We link the noise to the character's position 'i'
let ctx = SignalContext::new(0, 0).with_char_index(i);
let n = noise.sample_with_context(0.0, &ctx);
// 3. Threshold Logic
// Progress + Noise > Threshold = Decrypted
// This creates the "ragged" reveal effect
if p * 1.5 + n * 0.5 > 1.2 {
    show_decrypted_char(target_char);
}
"#;
    struct DecryptionLine {
        text: String,
        progress: Ramp,
        start_time: f64,
        use_full_ascii: bool,
        pulse_on_decrypt: bool,
    }
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // --- Terminal Setup ---
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // --- Real-time audio setup ---
        #[cfg(feature = "realtime-audio")]
        let audio_state = Arc::new(SharedDecryptState::new());
        #[cfg(feature = "realtime-audio")]
        let (_stream, audio_available) = {
            let state = Arc::clone(&audio_state);
            match OutputStream::try_default() {
                Ok((stream, handle)) => {
                    let source = DecryptAudioSource::new(48000, state);
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
        let noise = PerCharacterNoise::new(0xCAFEBABE, 1.0, 0.0);
        let shimmer = WhiteNoise::new(42, 1.0, 15.0);
        let pulse_signal = Sine::new(0.2, 1.0, 0.0, 0.0);
        let prompt_pulse = Sine::new(1.0, 1.0, 0.0, 0.0);
        let lines_data = [("ESTABLISHING SECURE CONNECTION...", false, false),
            ("HANDSHAKE PROTOCOL: VERIFIED", false, false),
            ("DOWNLOADING PAYLOAD: MIXED-SIGNALS.CRATE", true, false),
            ("ACCESS GRANTED - SYSTEM UNLOCKED", true, true)];
        let mut active_lines: Vec<DecryptionLine> = Vec::new();
        let mut max_end_time: f64 = 0.0;
        for (i, (text, full_ascii, pulse)) in lines_data.iter().enumerate() {
            let start = i as f64 * 1.5;
            let duration = 2.0;
            active_lines.push(DecryptionLine {
                text: text.to_string(),
                progress: Ramp::new(0.0, 1.2, duration as f32),
                start_time: start,
                use_full_ascii: *full_ascii,
                pulse_on_decrypt: *pulse,
            });
            max_end_time = max_end_time.max(start + duration);
        }
        let completion_time = max_end_time + 1.0;
        let mut app_time = 0.0;
        let mut show_code = false;
        let symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        let ascii: String = (33..127).map(char::from).collect();
        loop {
            terminal.draw(|f| {
                let area = f.area();
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(22),
                        Constraint::Min(1),
                    ])
                    .split(area);
                let center_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(80),
                        Constraint::Min(1),
                    ])
                    .split(vertical_chunks[1])[1];
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(8), // Header
                        Constraint::Min(0),    // Terminal
                    ])
                    .split(center_area);
                // --- Educational Header ---
                let explanation = vec![
                    Line::from(vec![
                        Span::styled(
                            "HOW IT WORKS: ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("We use "),
                        Span::styled("PerCharacterNoise", Style::default().fg(Color::Green)),
                        Span::raw(" to give each character a stable random value."),
                    ]),
                    Line::from(vec![
                        Span::raw("1. "),
                        Span::styled(
                            "SignalContext::with_char_index(i)",
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(" ensures the noise is deterministic per character."),
                    ]),
                    Line::from(vec![
                        Span::raw("2. The "),
                        Span::styled("PULSING", Style::default().fg(Color::Green)),
                        Span::raw(" effect is a "),
                        Span::styled("Sine(0.2Hz)", Style::default().fg(Color::Cyan)),
                        Span::raw(" signal sampled at current time."),
                    ]),
                    Line::from({
                        let mut spans = vec![
                            Span::styled("Space", Style::default().fg(Color::Yellow)),
                            Span::raw(" restart | "),
                            Span::styled("'c'", Style::default().fg(Color::Yellow)),
                            Span::raw(" code | "),
                        ];
                        if audio_available {
                            #[cfg(feature = "realtime-audio")]
                            {
                                let mute_status = if audio_state.is_muted() {
                                    "[MUTED]"
                                } else {
                                    "[♪]"
                                };
                                spans.push(Span::styled(
                                    mute_status,
                                    Style::default().fg(Color::Green),
                                ));
                                spans.push(Span::raw(" "));
                                spans.push(Span::styled("'s'", Style::default().fg(Color::Yellow)));
                                spans.push(Span::raw(" mute | "));
                            }
                        }
                        spans.push(Span::styled("'q'", Style::default().fg(Color::Yellow)));
                        spans.push(Span::raw(" quit"));
                        spans
                    }),
                ];
                let header_block = Paragraph::new(explanation)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" API Usage ")
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
                    .wrap(Wrap { trim: true });
                f.render_widget(header_block, chunks[0]);
                // --- Decryption Effect ---
                let mut text_lines = Vec::new();
                text_lines.push(Line::from(""));
                let pulse_val = pulse_signal.sample(app_time);
                let pulse_green = (pulse_val * 225.0 + 30.0) as u8;
                let pulse_color = Color::Rgb(0, pulse_green, 0);
                #[cfg(feature = "realtime-audio")]
                let mut cycling_count: u32 = 0;
                #[cfg(feature = "realtime-audio")]
                let mut last_cycling_char: u32 = 0;

                for line_data in &active_lines {
                    let local_t = (app_time - line_data.start_time).max(0.0);
                    let p = line_data.progress.sample(local_t);
                    let mut spans = Vec::new();
                    let chars: Vec<char> = line_data.text.chars().collect();
                    let scramble_set = if line_data.use_full_ascii {
                        &ascii
                    } else {
                        symbols
                    };
                    for (i, target_char) in chars.iter().enumerate() {
                        let ctx = SignalContext::new(0, 0).with_char_index(i);
                        let n = noise.sample_with_context(0.0, &ctx);
                        if p * 1.5 + n * 0.5 > 1.2 {
                            let style = if line_data.pulse_on_decrypt {
                                Style::default()
                                    .fg(pulse_color)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD)
                            };
                            spans.push(Span::styled(target_char.to_string(), style));
                        } else if p > 0.1 {
                            let shimmer_val = shimmer.sample(app_time + i as f64 * 0.1);
                            let idx = (shimmer_val * scramble_set.len() as f32) as usize
                                % scramble_set.len();
                            let cycling_char = scramble_set.chars().nth(idx).unwrap();
                            let char_str = cycling_char.to_string();
                            spans
                                .push(Span::styled(char_str, Style::default().fg(Color::DarkGray)));
                            // Track cycling for audio
                            #[cfg(feature = "realtime-audio")]
                            {
                                cycling_count += 1;
                                last_cycling_char = cycling_char as u32;
                            }
                        } else {
                            spans.push(Span::raw(" "));
                        }
                    }
                    text_lines.push(Line::from(spans));
                    text_lines.push(Line::from(""));
                }

                // Update audio state with cycling character info
                #[cfg(feature = "realtime-audio")]
                if audio_available {
                    audio_state.set_char(last_cycling_char);
                    audio_state.set_active(cycling_count);
                }
                if app_time > completion_time {
                    let prompt_val = prompt_pulse.sample(app_time);
                    let brightness = (prompt_val * 200.0 + 55.0) as u8;
                    let color = Color::Rgb(brightness, brightness, brightness);
                    text_lines.push(Line::from(""));
                    text_lines.push(Line::from(Span::styled(
                        "PRESS SPACE TO RESTART DEMO",
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    )));
                }
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Secure Terminal ")
                    .style(Style::default().fg(Color::Green));
                let paragraph = Paragraph::new(text_lines)
                    .block(block)
                    .alignment(Alignment::Left);
                f.render_widget(paragraph, chunks[1]);
                // --- Code Overlay ---
                if show_code {
                    let popup_area = centered_rect(60, 50, area);
                    f.render_widget(Clear, popup_area); // Clear background
                    let code_block = Paragraph::new(CODE_SNIPPET)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Code Snippet ")
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
                        KeyCode::Char(' ') => app_time = 0.0,
                        KeyCode::Char('c') => show_code = !show_code,
                        KeyCode::Char('s') =>
                        {
                            #[cfg(feature = "realtime-audio")]
                            if audio_available {
                                audio_state.toggle_mute();
                            }
                        }
                        _ => {}
                    }
                }
            }
            app_time += 0.016;
        }
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
    // Helper to center the popup
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

// <FILE>examples/decryption.rs</FILE> - <DESC>Demonstrates per-character deterministic noise with Ratatui</DESC>
// <VERS>END OF VERSION: 2.0.0 - 2026-01-02</VERS>
