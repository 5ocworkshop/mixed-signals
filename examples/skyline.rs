// <FILE>examples/skyline.rs</FILE> - <DESC>Demonstrates procedural generation with Perlin and Quantize</DESC>
// <VERS>VERSION: 1.5.0 - 2025-12-27T13:48:00Z</VERS>
// <WCTX>Add code snippet overlay</WCTX>
// <CLOG>Added 'c' key handler and code snippet modal popup</CLOG>

//! # Procedural Skyline Example (Ratatui)
//!
//! Demonstrates:
//! 1. `PerlinNoise`: Generating smooth terrain/skyline.
//! 2. `Quantize`: Snapping noise to discrete building heights.
//! 3. `SignalExt`: Fluent composition.
//!
//! **Note:** Ratatui is OPTIONAL.
//!
//! Run with: `cargo run --example skyline --features visualization`
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
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
        Terminal,
    };
    use std::io;
    const CODE_SNIPPET: &str = r#"
// 1. Generate Terrain
// Low frequency Perlin noise for general shape
let terrain = PerlinNoise::new(42, 0.05, 1.0);
// 2. Quantize into Buildings
// Snap smooth noise to 10 discrete levels
let buildings = Quantize::new(terrain, 10);
// 3. Render Loop
// Use x coordinate as time input
let h = buildings.sample(scroll_x + x as f64);
// 4. Procedural Windows (Stateless)
// Hash coords to decide if window is lit
let hash = (x * 37) ^ (row * 13) ^ (scroll_x as usize);
let is_window = hash % 7 == 0;
"#;
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        // 1. The Skyline Generator
        let terrain = PerlinNoise::new(42, 0.05, 1.0);
        let buildings = Quantize::new(terrain, 10);
        let mut scroll_x = 0.0;
        let mut show_code = false;
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
                        Constraint::Length(8), // Increased header height
                        Constraint::Min(0),
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
                        Span::raw("We treat the X coordinate as 'time' to sample the noise field."),
                    ]),
                    Line::from(vec![
                        Span::raw("1. "),
                        Span::styled("PerlinNoise", Style::default().fg(Color::Green)),
                        Span::raw(" generates a smooth, continuous curve (the terrain)."),
                    ]),
                    Line::from(vec![
                        Span::raw("2. "),
                        Span::styled(
                            "Quantize::new(signal, 10)",
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(" snaps that curve into 10 discrete height steps."),
                    ]),
                    Line::from(vec![Span::raw(
                        "3. Windows are procedural! We hash (x, y, scroll) coordinates to decide",
                    )]),
                    Line::from(vec![
                        Span::styled("Space", Style::default().fg(Color::Yellow)),
                        Span::raw(" to reset | "),
                        Span::styled("'c'", Style::default().fg(Color::Yellow)),
                        Span::raw(" to show code | "),
                        Span::styled("'q'", Style::default().fg(Color::Yellow)),
                        Span::raw(" to quit"),
                    ]),
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
                // --- City Rendering ---
                let inner_area = chunks[1];
                let view_area = Rect::new(
                    inner_area.x + 1,
                    inner_area.y + 1,
                    inner_area.width - 2,
                    inner_area.height - 2,
                );
                let mut buffer =
                    vec![vec![' '; view_area.width as usize]; view_area.height as usize];
                for x in 0..view_area.width {
                    let t = scroll_x + x as f64;
                    let h = buildings.sample(t);
                    let building_height = (h * view_area.height as f32) as usize;
                    for y in 0..building_height {
                        let row = view_area.height as usize - 1 - y;
                        if row < buffer.len() {
                            let hash = (x as usize * 37) ^ (row * 13) ^ (scroll_x as usize);
                            let is_window = hash % 7 == 0 && y < building_height - 1;
                            buffer[row][x as usize] = if is_window { '⚬' } else { '█' };
                        }
                    }
                }
                for (i, row) in buffer.iter().enumerate() {
                    let line: String = row.iter().collect();
                    let p = Paragraph::new(line).style(Style::default().fg(Color::Blue));
                    f.render_widget(
                        p,
                        Rect::new(view_area.x, view_area.y + i as u16, view_area.width, 1),
                    );
                }
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Procedural Skyline ")
                    .style(Style::default().fg(Color::White));
                f.render_widget(block, inner_area);
                // --- Code Overlay ---
                if show_code {
                    let popup_area = centered_rect(60, 50, area);
                    f.render_widget(Clear, popup_area);
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
            if event::poll(std::time::Duration::from_millis(32))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => scroll_x = 0.0,
                        KeyCode::Char('c') => show_code = !show_code,
                        _ => {}
                    }
                }
            }
            scroll_x += 0.2;
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

// <FILE>examples/skyline.rs</FILE> - <DESC>Demonstrates procedural generation with Perlin and Quantize</DESC>
// <VERS>END OF VERSION: 1.5.0 - 2025-12-27T13:48:00Z</VERS>
