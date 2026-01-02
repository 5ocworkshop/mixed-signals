// <FILE>examples/snow_demo.rs</FILE> - <DESC>Snow demo (mixed-signals, terminal)</DESC>
// <VERS>VERSION: 0.1.0</VERS>
// <WCTX>mixed-signals examples</WCTX>
// <CLOG>Lightweight snow effect with selectable modes and signal-driven motion</CLOG>

//! # Snow Demo (Mixed-Signals)
//!
//! A lightweight terminal snow effect using mixed-signals. No sixel sprites.
//!
//! Usage:
//! ```bash
//! cargo run --example snow_demo --features visualization
//! ```

#[cfg(feature = "visualization")]
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "visualization")]
use mixed_signals::prelude::*;
#[cfg(feature = "visualization")]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
#[cfg(feature = "visualization")]
use std::io;

#[cfg(feature = "visualization")]
const DEFAULT_FLAKES: usize = 200;
#[cfg(feature = "visualization")]
const MAX_FLAKES: usize = 400;
#[cfg(feature = "visualization")]
const MIN_FLAKES: usize = 10;
#[cfg(feature = "visualization")]
const MAX_ACTIVE_FLAKES: usize = 900;

#[cfg(feature = "visualization")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Drift,
    Gust,
    Swirl,
    Blizzard,
    HeavySnowfall,
}

#[cfg(feature = "visualization")]
impl Mode {
    fn name(&self) -> &'static str {
        match self {
            Mode::Drift => "Drift",
            Mode::Gust => "Gust",
            Mode::Swirl => "Swirl",
            Mode::Blizzard => "Blizzard",
            Mode::HeavySnowfall => "Heavy Snowfall",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Mode::Drift => "Steady fall with light jitter",
            Mode::Gust => "Periodic gusts push flakes sideways",
            Mode::Swirl => "Swirl + gentle oscillation",
            Mode::Blizzard => "Fast gusts with chaotic crosswind",
            Mode::HeavySnowfall => "Ramping density with noisy bursts",
        }
    }

    fn signals(&self) -> &'static str {
        match self {
            Mode::Drift => "Signals: Sine(drift) + CorrelatedNoise(jitter)",
            Mode::Gust => "Signals: Sine(gust) + CorrelatedNoise(jitter)",
            Mode::Swirl => "Signals: Sine(swirl) + CorrelatedNoise(jitter)",
            Mode::Blizzard => {
                "Signals: Sine(fast gust) + WhiteNoise(turbulence) + CorrelatedNoise(jitter)"
            }
            Mode::HeavySnowfall => {
                "Signals: Ramp(density) + WhiteNoise(hf) + Sine(drift) + CorrelatedNoise(jitter)"
            }
        }
    }
}

#[cfg(feature = "visualization")]
#[derive(Clone)]
struct Flake {
    x: f32,
    y: f32,
    size: f32,
    seed: u64,
}

#[cfg(feature = "visualization")]
fn clamp_index(v: f32, max: u16) -> u16 {
    v.round().clamp(0.0, (max.saturating_sub(1)) as f32) as u16
}

#[cfg(feature = "visualization")]
fn spawn_flakes(count: usize, width: u16, height: u16) -> Vec<Flake> {
    let mut flakes = Vec::with_capacity(count);
    for i in 0..count {
        let seed = 1000 + i as u64 * 17;
        let rng = SeededRandom::new(seed, 1.0, 0.0);
        let x = rng.sample(i as f64) * width as f32;
        let y = rng.sample(i as f64 * 0.37) * height as f32;
        let size = 0.6 + (rng.sample(i as f64 * 0.73) * 0.8);
        flakes.push(Flake { x, y, size, seed });
    }
    flakes
}

#[cfg(feature = "visualization")]
fn flake_char(size: f32, _seed: u64, _t: f32) -> char {
    if size < 0.8 {
        '.'
    } else if size < 1.1 {
        '*'
    } else {
        'o'
    }
}

#[cfg(feature = "visualization")]
fn render_ui(
    frame: &mut ratatui::Frame,
    area: Rect,
    mode: Mode,
    speed: f32,
    flakes: usize,
    active_flakes: usize,
) {
    let flake_line = if active_flakes == flakes {
        format!("Speed: {:.2} | Flakes: {}", speed, flakes)
    } else {
        format!(
            "Speed: {:.2} | Flakes: {} (active: {})",
            speed, flakes, active_flakes
        )
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            "mixed-signals Snow Demo (no sprites)",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            mode.signals(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Mode: {} — {}", mode.name(), mode.description()),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            flake_line,
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "Controls: q quit | +/- flakes | ←→ speed | 1 drift | 2 gust | 3 swirl | 4 blizzard | 5 heavy",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "Symbols: . * o (small → large)",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

#[cfg(not(feature = "visualization"))]
fn main() {
    eprintln!("snow_demo requires the visualization feature: cargo run --example snow_demo --features visualization");
}

#[cfg(feature = "visualization")]
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

#[cfg(feature = "visualization")]
impl TerminalGuard {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(err) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err.into());
        }
        let backend = CrosstermBackend::new(stdout);
        match Terminal::new(backend) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(err) => {
                let _ = disable_raw_mode();
                let mut stdout = io::stdout();
                let _ = execute!(stdout, LeaveAlternateScreen);
                Err(err.into())
            }
        }
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

#[cfg(feature = "visualization")]
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[cfg(feature = "visualization")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = TerminalGuard::new()?;
    let terminal = guard.terminal_mut();

    let mut mode = Mode::Drift;
    let mut speed = 1.0f32;
    let mut flakes_count = DEFAULT_FLAKES;
    let mut time = 0.0f64;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Min(1)])
                .split(size);

            let area = chunks[1];
            if area.width == 0 || area.height == 0 {
                return;
            }
            let drift = Sine::new(0.3, 0.6, 0.0, 0.0);
            let gust = Sine::new(0.9, 1.0, 0.0, 0.0);
            let swirl = Sine::new(0.6, 1.0, 0.0, 0.25);
            let blizzard = Sine::new(2.1, 1.0, 0.0, 0.0);
            let turbulence = WhiteNoise::new(9001, 1.0, 220.0);
            let density_ramp = Ramp::new(0.2, 1.0, 12.0);
            let density_noise = WhiteNoise::new(4242, 1.0, 180.0);
            let jitter = CorrelatedNoise::new(42, 0.9, 1.0, 0.0).unwrap();

            let active_flakes = if mode == Mode::HeavySnowfall {
                let ramp = density_ramp.sample(time * 0.08);
                let burst = density_noise.sample(time * 1.6);
                let density = (0.4 + 0.6 * ramp) * (0.7 + 0.3 * burst);
                let scaled = flakes_count as f32 * (1.0 + density * 2.2);
                scaled.round() as usize
            } else {
                flakes_count
            };
            let time_f32 = time as f32; // For f32 arithmetic below
            let active_flakes = active_flakes.clamp(MIN_FLAKES, MAX_ACTIVE_FLAKES);

            render_ui(f, chunks[0], mode, speed, flakes_count, active_flakes);

            let mut buffer = vec![vec![' '; area.width as usize]; area.height as usize];
            let flakes = spawn_flakes(active_flakes, area.width, area.height);
            let fall_speed = match mode {
                Mode::Blizzard => 0.9,
                Mode::HeavySnowfall => 0.7,
                _ => 0.6,
            };

            for (i, flake) in flakes.iter().enumerate() {
                let ctx = SignalContext::new(i as u64, flake.seed);
                let j =
                    (jitter.sample_with_context(time * 0.7 + i as f64 * 0.01, &ctx) - 0.5) * 0.4;
                let fall = (time_f32 * fall_speed * speed + flake.y).rem_euclid(area.height as f32);

                let mode_x = match mode {
                    Mode::Drift => drift.sample(time + flake.seed as f64 * 0.001),
                    Mode::Gust => gust.sample(time + flake.seed as f64 * 0.002),
                    Mode::Swirl => swirl.sample(time + flake.seed as f64 * 0.002),
                    Mode::Blizzard => {
                        let fast = blizzard.sample(time * 1.4 + flake.seed as f64 * 0.004);
                        let chaos = turbulence.sample(time * 2.2 + i as f64 * 0.05);
                        (fast * 0.6 + chaos * 0.4).clamp(0.0, 1.0)
                    }
                    Mode::HeavySnowfall => drift.sample(time + flake.seed as f64 * 0.0008),
                };
                let x = (flake.x + (mode_x - 0.5) * 6.0 + j * 4.0).rem_euclid(area.width as f32);

                let xi = clamp_index(x, area.width) as usize;
                let yi = clamp_index(fall, area.height) as usize;
                buffer[yi][xi] = flake_char(flake.size, flake.seed, time_f32);
            }

            for (row_idx, row) in buffer.iter().enumerate() {
                let line: String = row.iter().collect();
                let y = area.y + row_idx as u16;
                let paragraph = Paragraph::new(line).style(Style::default().fg(Color::White));
                f.render_widget(paragraph, Rect::new(area.x, y, area.width, 1));
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => speed = (speed - 0.1).max(0.2),
                    KeyCode::Right => speed = (speed + 0.1).min(3.0),
                    KeyCode::Char('+') => flakes_count = (flakes_count + 10).min(MAX_FLAKES),
                    KeyCode::Char('-') => {
                        flakes_count = flakes_count.saturating_sub(10).max(MIN_FLAKES)
                    }
                    KeyCode::Char('1') => mode = Mode::Drift,
                    KeyCode::Char('2') => mode = Mode::Gust,
                    KeyCode::Char('3') => mode = Mode::Swirl,
                    KeyCode::Char('4') => mode = Mode::Blizzard,
                    KeyCode::Char('5') => mode = Mode::HeavySnowfall,
                    _ => {}
                }
            }
        }

        time += 0.033;
    }

    Ok(())
}

// <FILE>examples/snow_demo.rs</FILE> - <DESC>Snow demo (mixed-signals, terminal)</DESC>
// <VERS>END OF VERSION: 0.1.0</VERS>
