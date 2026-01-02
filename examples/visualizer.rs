// <FILE>examples/visualizer.rs</FILE> - <DESC>Interactive signal visualizer demonstration</DESC>
// <VERS>VERSION: 1.6.0 - 2025-12-27T13:14:12Z</VERS>
// <WCTX>Upgrade time precision to f64</WCTX>
// <CLOG>Updated time variables and SignalEntry time_span to f64</CLOG>

//! # Signal Visualizer
//!
//! Interactive terminal application demonstrating the SignalView widget.
//! Shows various signal types plotted in real-time with color-coded waveforms.
//! Uses btop-style Braille filled area graphs for smooth, high-resolution rendering.
//!
//! ## Usage
//! ```bash
//! cargo run --example visualizer --features visualization
//! ```
//!
//! Press 'q' to quit.
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mixed_signals::{prelude::*, visualization::ColorGradient};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SignalCategory {
    Waveforms,
    Noise,
    Envelopes,
    CompositionProcessing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimeMode {
    Moving,
    Static,
    Loop,
}
struct SignalEntry {
    name: &'static str,
    color: Color,
    make: fn() -> Box<dyn Signal>,
    time_span: f64,
    time_mode: TimeMode,
    category: SignalCategory,
}
fn signal_catalog() -> Vec<SignalEntry> {
    vec![
        // Generators
        SignalEntry {
            name: "Sine",
            color: Color::Blue,
            make: || Box::new(Sine::new(2.0, 1.0, 0.0, 0.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Triangle",
            color: Color::Green,
            make: || Box::new(Triangle::new(2.0, 1.0, 0.0, 0.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Square",
            color: Color::Yellow,
            make: || Box::new(Square::new(2.0, 1.0, 0.0, 0.0, 0.5)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Sawtooth",
            color: Color::Magenta,
            make: || Box::new(Sawtooth::new(2.0, 1.0, 0.0, 0.0, false)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Pulse",
            color: Color::LightBlue,
            make: || Box::new(Pulse::window(0.65, 0.85)),
            time_span: 1.0,
            time_mode: TimeMode::Loop,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Step",
            color: Color::LightGreen,
            make: || Box::new(Step::new(0.0, 1.0, 0.35)),
            time_span: 1.0,
            time_mode: TimeMode::Loop,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Ramp",
            color: Color::LightYellow,
            make: || Box::new(Ramp::new(0.0, 1.0, 1.0)),
            time_span: 1.0,
            time_mode: TimeMode::Loop,
            category: SignalCategory::Waveforms,
        },
        SignalEntry {
            name: "Constant",
            color: Color::Gray,
            make: || Box::new(Constant::new(0.5)),
            time_span: 2.0,
            time_mode: TimeMode::Static,
            category: SignalCategory::Waveforms,
        },
        // Noise & Random
        SignalEntry {
            name: "White Noise",
            color: Color::Red,
            make: || Box::new(WhiteNoise::new(42, 1.0, 60.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Perlin Noise",
            color: Color::Cyan,
            make: || Box::new(PerlinNoise::with_seed(99).with_octaves(4, 0.5)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Seeded Random",
            color: Color::LightRed,
            make: || Box::new(SeededRandom::with_seed(7)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Gaussian Noise",
            color: Color::LightMagenta,
            make: || Box::new(GaussianNoise::new(7, 0.5, 0.15, 0.0).unwrap()),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Poisson Noise",
            color: Color::LightCyan,
            make: || Box::new(PoissonNoise::new(7, 2.0, 1.0, 0.0).unwrap()),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Correlated Noise",
            color: Color::LightBlue,
            make: || Box::new(CorrelatedNoise::new(7, 0.95, 1.0, 0.0).unwrap()),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Pink Noise",
            color: Color::LightGreen,
            make: || Box::new(PinkNoise::new(7, 1.0, 0.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Spatial Noise",
            color: Color::LightYellow,
            make: || Box::new(SpatialNoise::new(7, 1.0, 1.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        SignalEntry {
            name: "Per-Character Noise",
            color: Color::LightMagenta,
            make: || Box::new(PerCharacterNoise::new(7, 1.0, 0.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::Noise,
        },
        // Envelopes
        SignalEntry {
            name: "ADSR",
            color: Color::Green,
            make: || Box::new(Adsr::new(0.2, 0.3, 0.7, 0.4)),
            time_span: 1.0,
            time_mode: TimeMode::Loop,
            category: SignalCategory::Envelopes,
        },
        SignalEntry {
            name: "Linear Envelope",
            color: Color::LightGreen,
            make: || Box::new(LinearEnvelope::new(0.2, 0.2)),
            time_span: 1.0,
            time_mode: TimeMode::Loop,
            category: SignalCategory::Envelopes,
        },
        SignalEntry {
            name: "Impact",
            color: Color::Red,
            make: || Box::new(Impact::new(1.0, 3.0)),
            time_span: 1.0,
            time_mode: TimeMode::Loop,
            category: SignalCategory::Envelopes,
        },
        // Composition
        SignalEntry {
            name: "Add",
            color: Color::Blue,
            make: || Box::new(Add::new(Sine::with_frequency(1.0), Constant::new(0.2))),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Multiply",
            color: Color::Cyan,
            make: || {
                Box::new(Multiply::new(
                    Sine::with_frequency(1.0),
                    Sine::with_frequency(3.0),
                ))
            },
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Mix",
            color: Color::Magenta,
            make: || {
                Box::new(Mix::new(
                    Sine::with_frequency(1.0),
                    Triangle::with_frequency(0.5),
                    0.5,
                ))
            },
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Frequency Mod",
            color: Color::Yellow,
            make: || {
                Box::new(FrequencyMod::new(
                    Sine::with_frequency(1.0),
                    Sine::with_frequency(0.5),
                    1.5,
                    1.0,
                ))
            },
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        // Processing
        SignalEntry {
            name: "Abs",
            color: Color::Blue,
            make: || Box::new(Abs::new(Sine::with_frequency(1.0))),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Invert",
            color: Color::Green,
            make: || Box::new(Invert::new(Sine::with_frequency(1.0))),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Clamp",
            color: Color::Yellow,
            make: || Box::new(Clamp::new(Sine::with_frequency(1.0), 0.2, 0.8)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Remap",
            color: Color::Magenta,
            make: || Box::new(Remap::new(Sine::with_frequency(1.0), 0.0, 1.0, -1.0, 1.0)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
        SignalEntry {
            name: "Quantize",
            color: Color::Cyan,
            make: || Box::new(Quantize::new(Sine::with_frequency(1.0), 6)),
            time_span: 2.0,
            time_mode: TimeMode::Moving,
            category: SignalCategory::CompositionProcessing,
        },
    ]
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Run the app
    let res = run_app(&mut terminal);
    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    if let Err(err) = res {
        println!("Error: {:?}", err);
    }
    Ok(())
}
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    let mut time: f64 = 0.0;
    let mut paused = false;
    let mut scroll_speed: f64 = 0.002;
    let mut use_gradients = false;
    let mut selected_graph: usize = 0;
    let mut fullscreen = false;
    const MAX_SPEED: f64 = 0.006;
    const MIN_SPEED: f64 = 0.0004;
    const SPEED_STEP: f64 = 0.0004;
    const PAGE_SIZE: usize = 8;
    let mut filter: Option<SignalCategory> = None;
    let catalog = signal_catalog();
    loop {
        let filtered: Vec<&SignalEntry> = match filter {
            Some(category) => catalog
                .iter()
                .filter(|entry| entry.category == category)
                .collect(),
            None => catalog.iter().collect(),
        };
        let filtered_len = filtered.len().max(1);
        if selected_graph >= filtered_len {
            selected_graph = filtered_len.saturating_sub(1);
        }
        let page = selected_graph / PAGE_SIZE;
        let page_count = filtered_len.div_ceil(PAGE_SIZE);
        let page_start = page * PAGE_SIZE;
        let page_end = (page_start + PAGE_SIZE).min(filtered_len);
        let visible = &filtered[page_start..page_end];
        terminal.draw(|f| {
            let size = f.area();
            // Create layout - either fullscreen or normal multi-graph view
            let chunks = if fullscreen {
                // Fullscreen mode: title + selected graph only
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(5),      // Title (3 lines + borders)
                        Constraint::Percentage(100), // Selected graph (fullscreen)
                    ])
                    .split(size)
            } else {
                // Normal mode: current page of graphs
                let mut constraints = Vec::with_capacity(1 + visible.len());
                constraints.push(Constraint::Length(5));
                let ratio = visible.len().max(1) as u32;
                for _ in 0..visible.len() {
                    constraints.push(Constraint::Ratio(1, ratio));
                }
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(size)
            };
            // Title
            let status = if paused { " [PAUSED]" } else { "" };
            let gradient_status = if use_gradients { " [GRADIENTS]" } else { "" };
            let fullscreen_status = if fullscreen { " [FULLSCREEN]" } else { "" };
            let speed_pct = (scroll_speed / MAX_SPEED * 100.0) as u8;
            let selected_name = filtered.get(selected_graph).map(|e| e.name).unwrap_or("None");
            let title = Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("mixed-signals Visualizer{}{}{} | Speed: {}% | Selected: {}",
                        status, gradient_status, fullscreen_status, speed_pct, selected_name),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "Note: Enter toggles fullscreen | Space pauses",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    format!("q:quit | ←→:speed | c:gradients | ↑↓:select | 1:all 2:waves 3:noise 4:env 5:mix/proc | Page {}/{}",
                        page + 1, page_count),
                    Style::default().fg(Color::Gray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);
            // Bottom-right viewing indicator (with down-arrow nudge)
            if size.width > 12 {
                let view_text = format!("viewing {}/{} ↓", selected_graph + 1, filtered_len);
                let x = size.width.saturating_sub(view_text.len() as u16 + 2);
                let y = size.height.saturating_sub(2);
                let view = Paragraph::new(Line::from(Span::styled(
                    view_text.as_str(),
                    Style::default().fg(Color::Gray),
                )));
                f.render_widget(view, Rect::new(x, y, view_text.len() as u16 + 1, 1));
            }
            // Render graphs based on mode (fullscreen or normal)
            if fullscreen {
                let entry = filtered[selected_graph];
                let signal = (entry.make)();
                render_signal(
                    f,
                    chunks[1],
                    RenderConfig {
                        signal: signal.as_ref(),
                        title: entry.name,
                        color: entry.color,
                        current_time: time,
                        time_span: entry.time_span,
                        time_mode: entry.time_mode,
                        use_gradients,
                        is_selected: true,
                    },
                );
            } else {
                for (i, entry) in visible.iter().enumerate() {
                    let signal = (entry.make)();
                    let index = page_start + i;
                    render_signal(
                        f,
                        chunks[i + 1],
                        RenderConfig {
                            signal: signal.as_ref(),
                            title: entry.name,
                            color: entry.color,
                            current_time: time,
                            time_span: entry.time_span,
                            time_mode: entry.time_mode,
                            use_gradients,
                            is_selected: index == selected_graph,
                        },
                    );
                }
            }
        })?;
        // Handle input
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('c') => use_gradients = !use_gradients,
                    KeyCode::Left => {
                        // Speed up (increase scroll_speed toward max)
                        scroll_speed = (scroll_speed + SPEED_STEP).min(MAX_SPEED);
                    }
                    KeyCode::Right => {
                        // Slow down (decrease scroll_speed toward min)
                        scroll_speed = (scroll_speed - SPEED_STEP).max(MIN_SPEED);
                    }
                    KeyCode::Up => {
                        selected_graph = if selected_graph == 0 {
                            filtered_len - 1
                        } else {
                            selected_graph - 1
                        };
                    }
                    KeyCode::Down => {
                        selected_graph = (selected_graph + 1) % filtered_len;
                    }
                    KeyCode::Enter => {
                        // Toggle fullscreen mode
                        fullscreen = !fullscreen;
                    }
                    KeyCode::Char('1') => {
                        filter = None;
                        selected_graph = 0;
                    }
                    KeyCode::Char('2') => {
                        filter = Some(SignalCategory::Waveforms);
                        selected_graph = 0;
                    }
                    KeyCode::Char('3') => {
                        filter = Some(SignalCategory::Noise);
                        selected_graph = 0;
                    }
                    KeyCode::Char('4') => {
                        filter = Some(SignalCategory::Envelopes);
                        selected_graph = 0;
                    }
                    KeyCode::Char('5') => {
                        filter = Some(SignalCategory::CompositionProcessing);
                        selected_graph = 0;
                    }
                    _ => {}
                }
            }
        }
        // Advance time (simulate animation) unless paused
        if !paused {
            time += scroll_speed;
            if time > 2.0 {
                time = 0.0; // Reset for looping
            }
        }
    }
}
struct RenderConfig<'a> {
    signal: &'a dyn Signal,
    title: &'a str,
    color: Color,
    current_time: f64,
    time_span: f64,
    time_mode: TimeMode,
    use_gradients: bool,
    is_selected: bool,
}

fn render_signal(f: &mut ratatui::Frame, area: Rect, cfg: RenderConfig<'_>) {
    let RenderConfig {
        signal,
        title,
        color,
        current_time,
        time_span,
        time_mode,
        use_gradients,
        is_selected,
    } = cfg;
    // Render block with title - white border if selected
    let border_color = if is_selected { Color::White } else { color };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    f.render_widget(block, area);
    // Render signal within the block
    // Braille mode is the default, providing 2×4 sub-cell resolution
    let range = signal.output_range();
    if matches!(time_mode, TimeMode::Loop) {
        struct LoopSignal<'a> {
            inner: &'a dyn Signal,
            phase: f64,
            period: f64,
        }
        impl<'a> Signal for LoopSignal<'a> {
            fn output_range(&self) -> SignalRange {
                self.inner.output_range()
            }
            fn sample(&self, t: f64) -> f32 {
                let wrapped = (t + self.phase).rem_euclid(self.period);
                self.inner.sample(wrapped)
            }
            fn sample_with_context(&self, t: f64, ctx: &SignalContext) -> f32 {
                let wrapped = (t + self.phase).rem_euclid(self.period);
                self.inner.sample_with_context(wrapped, ctx)
            }
        }
        let looped = LoopSignal {
            inner: signal,
            phase: current_time,
            period: time_span.max(0.0001),
        };
        let mut signal_view = SignalView::new(&looped)
            .time_range(0.0, time_span)
            .value_range(range.min, range.max);
        if use_gradients {
            let gradient = ColorGradient::new()
                .add_stop(0.0, Color::Blue)
                .add_stop(0.25, Color::Cyan)
                .add_stop(0.5, Color::Green)
                .add_stop(0.75, Color::Yellow)
                .add_stop(1.0, Color::Red);
            signal_view = signal_view.gradient(gradient);
        } else {
            signal_view = signal_view.style(Style::default().fg(color));
        }
        f.render_widget(signal_view, inner);
        return;
    }
    let start = match time_mode {
        TimeMode::Static => 0.0,
        TimeMode::Moving | TimeMode::Loop => current_time,
    };
    let mut signal_view = SignalView::new(signal)
        .time_range(start, start + time_span)
        .value_range(range.min, range.max);
    if use_gradients {
        let gradient = ColorGradient::new()
            .add_stop(0.0, Color::Blue)
            .add_stop(0.25, Color::Cyan)
            .add_stop(0.5, Color::Green)
            .add_stop(0.75, Color::Yellow)
            .add_stop(1.0, Color::Red);
        signal_view = signal_view.gradient(gradient);
    } else {
        signal_view = signal_view.style(Style::default().fg(color));
    }
    f.render_widget(signal_view, inner);
}

// <FILE>examples/visualizer.rs</FILE> - <DESC>Interactive signal visualizer demonstration</DESC>
// <VERS>END OF VERSION: 1.6.0 - 2025-12-27T13:14:12Z</VERS>
