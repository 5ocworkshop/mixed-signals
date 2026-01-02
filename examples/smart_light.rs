// <FILE>examples/smart_light.rs</FILE> - <DESC>Demonstrates signal composition for organic animation</DESC>
// <VERS>VERSION: 1.5.0 - 2025-12-27T13:48:00Z</VERS>
// <WCTX>Add code snippet overlay</WCTX>
// <CLOG>Added 'c' key handler and code snippet modal popup</CLOG>

//! # Smart Light Example (Ratatui)
//!
//! Demonstrates:
//! 1. Composition: Mixing Sine + Noise for organic "breathing".
//! 2. Fluent API: `.mix()`, `.scale()`.
//! 3. Interactive state changes.
//!
//! **Note:** Ratatui is OPTIONAL.
//!
//! Run with: `cargo run --example smart_light --features visualization`
#[cfg(not(feature = "visualization"))]
fn main() {
    println!("This example requires the 'visualization' feature.");
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
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
        Terminal,
    };
    use std::io;
    const CODE_SNIPPET: &str = r#"
// 1. Define Signals
let base = Sine::with_frequency(0.5);
let jitter = PerlinNoise::new(1, 2.0, 1.0);
// 2. Compose (Fluent API)
// Mix 80% Sine with 20% Noise
let signal = base.mix(jitter, 0.2);
// 3. Sample & Display
let val = signal.sample(time);
// Direct mapping to UI gauge
gauge.ratio(val.clamp(0.0, 1.0));
"#;
    enum DeviceState {
        Idle,
        Error,
    }
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        // --- Signal Definitions ---
        let idle_base = Sine::with_frequency(0.5);
        let idle_jitter = PerlinNoise::new(1, 2.0, 1.0);
        let idle_signal = idle_base.mix(idle_jitter, 0.2);
        let error_base = Square::new(4.0, 1.0, 0.0, 0.0, 0.5);
        let error_noise = WhiteNoise::new(99, 1.0, 60.0);
        let error_signal = error_base.mix(error_noise, 0.5);
        let mut state = DeviceState::Idle;
        let mut time = 0.0;
        let mut show_code = false;
        loop {
            terminal.draw(|f| {
                let area = f.area();
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(18),
                        Constraint::Min(1),
                    ])
                    .split(area);
                let center_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(60),
                        Constraint::Min(1),
                    ])
                    .split(vertical_chunks[1])[1];
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(8), // Header
                        Constraint::Length(3), // LED Gauge
                        Constraint::Min(0),    // Info
                    ])
                    .split(center_area);
                // --- Educational Header ---
                let explanation = vec![
                    Line::from(vec![
                        Span::styled("HOW IT WORKS: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("We use "),
                        Span::styled("composition", Style::default().fg(Color::Green)),
                        Span::raw(" to layer signals."),
                    ]),
                    Line::from(vec![
                        Span::raw("Idle Mode: "),
                        Span::styled("Sine::with_frequency(0.5).mix(PerlinNoise, 0.2)", Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(vec![
                        Span::raw("This mixes 80% perfect sine wave with 20% organic noise to create a 'living' breath effect."),
                    ]),
                ];
                let header_block = Paragraph::new(explanation)
                    .block(Block::default().borders(Borders::ALL).title(" API Usage ").border_style(Style::default().fg(Color::Yellow)))
                    .wrap(Wrap { trim: true });
                f.render_widget(header_block, chunks[0]);
                let (val, label, color) = match state {
                    DeviceState::Idle => {
                        (idle_signal.sample(time), "STATUS: ONLINE (Breathing)", Color::Cyan)
                    }
                    DeviceState::Error => {
                        (error_signal.sample(time), "STATUS: ERROR (Strobe)", Color::Red)
                    }
                };
                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title(" LED Output "))
                    .gauge_style(Style::default().fg(color))
                    .ratio(val.clamp(0.0, 1.0) as f64)
                    .label(format!("{:.2}", val));
                f.render_widget(gauge, chunks[1]);
                let info = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(format!("Current Mode: {}", label)),
                    Line::from(""),
                    Line::from("Controls:"),
                    Line::from("  [m]     Toggle Mode (Idle / Error)"),
                    Line::from("  [SPACE] Restart Demo"),
                    Line::from("  [c]     Show Code"),
                    Line::from("  [q]     Quit"),
                ]);
                f.render_widget(info, chunks[2]);
                // --- Code Overlay ---
                if show_code {
                    let popup_area = centered_rect(60, 50, area);
                    f.render_widget(Clear, popup_area);
                    let code_block = Paragraph::new(CODE_SNIPPET)
                        .block(Block::default().borders(Borders::ALL).title(" Code Snippet ").border_style(Style::default().fg(Color::Yellow)))
                        .style(Style::default().fg(Color::White));
                    f.render_widget(code_block, popup_area);
                }
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('m') => {
                            state = match state {
                                DeviceState::Idle => DeviceState::Error,
                                DeviceState::Error => DeviceState::Idle,
                            };
                        }
                        KeyCode::Char(' ') => {
                            time = 0.0; // Restart
                            state = DeviceState::Idle;
                        }
                        KeyCode::Char('c') => show_code = !show_code,
                        _ => {}
                    }
                }
            }
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

// <FILE>examples/smart_light.rs</FILE> - <DESC>Demonstrates signal composition for organic animation</DESC>
// <VERS>END OF VERSION: 1.5.0 - 2025-12-27T13:48:00Z</VERS>
