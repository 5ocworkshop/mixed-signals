// <FILE>src/visualization/cls_signal_view.rs</FILE> - <DESC>Signal oscilloscope widget</DESC>
// <VERS>VERSION: 1.3.0 - 2025-12-27T13:14:12Z</VERS>
// <WCTX>Upgrade time precision to f64</WCTX>
// <CLOG>Changed time_range and internal time math to f64 to support long-running animations</CLOG>

use crate::traits::Signal;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;
use std::cmp::Ordering;
/// btop-style Braille graph symbols: 5×5 lookup table
/// Row index = previous value (0-4), Column index = current value (0-4)
/// Creates filled area graph transitions
const BRAILLE_GRAPH_SYMBOLS: [[char; 5]; 5] = [
    [' ', '⢀', '⢠', '⢰', '⢸'],
    ['⡀', '⣀', '⣠', '⣰', '⣸'],
    ['⡄', '⣄', '⣤', '⣴', '⣼'],
    ['⡆', '⣆', '⣦', '⣶', '⣾'],
    ['⡇', '⣇', '⣧', '⣷', '⣿'],
];
/// Rendering mode for signal visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Use Braille characters for sub-cell resolution (2×4 dots per cell)
    Braille,
    /// Use simple block character
    Block,
}
/// Simple stepped color gradient.
#[derive(Debug, Clone)]
pub struct ColorGradient {
    stops: Vec<(f32, Color)>,
}
impl Default for ColorGradient {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorGradient {
    pub fn new() -> Self {
        Self { stops: Vec::new() }
    }
    pub fn add_stop(mut self, position: f32, color: Color) -> Self {
        let position = position.clamp(0.0, 1.0);
        self.stops.push((position, color));
        self.stops
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        self
    }
    fn color_for(&self, value: f32, range: (f32, f32)) -> Option<Color> {
        if self.stops.is_empty() {
            return None;
        }
        let (min, max) = range;
        let span = max - min;
        if span.abs() < 0.0001 {
            return Some(self.stops[self.stops.len() - 1].1);
        }
        let normalized = ((value - min) / span).clamp(0.0, 1.0);
        let mut last = self.stops[0].1;
        for (pos, color) in &self.stops {
            if normalized >= *pos {
                last = *color;
            } else {
                break;
            }
        }
        Some(last)
    }
}
/// ASCII oscilloscope widget for visualizing signals.
///
/// Renders a signal waveform within a given area, showing the signal's
/// value over time. Supports Braille rendering for higher resolution.
pub struct SignalView<'a> {
    signal: &'a dyn Signal,
    /// Time range to display (start, end)
    /// Uses f64 to support long-running animations without jitter
    time_range: (f64, f64),
    /// Value range to display (min, max)
    value_range: (f32, f32),
    /// Style for the waveform line
    style: Style,
    /// Rendering mode
    render_mode: RenderMode,
    /// Character to use for the waveform (Block mode only)
    wave_char: char,
    /// Show zero crossing line
    show_zero_line: bool,
    /// Style for zero line
    zero_line_style: Style,
    /// Optional gradient for value-based coloring
    gradient: Option<ColorGradient>,
}
impl<'a> SignalView<'a> {
    pub fn new(signal: &'a dyn Signal) -> Self {
        let range = signal.output_range();
        Self {
            signal,
            time_range: (0.0, 1.0),
            value_range: (range.min, range.max),
            style: Style::default().fg(Color::Cyan),
            render_mode: RenderMode::Braille,
            wave_char: '█',
            show_zero_line: true,
            zero_line_style: Style::default().fg(Color::DarkGray),
            gradient: None,
        }
    }
    pub fn time_range(mut self, start: f64, end: f64) -> Self {
        self.time_range = (start, end);
        self
    }
    pub fn value_range(mut self, min: f32, max: f32) -> Self {
        self.value_range = (min, max);
        self
    }
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }
    pub fn wave_char(mut self, c: char) -> Self {
        self.wave_char = c;
        self
    }
    pub fn show_zero_line(mut self, show: bool) -> Self {
        self.show_zero_line = show;
        self
    }
    pub fn zero_line_style(mut self, style: Style) -> Self {
        self.zero_line_style = style;
        self
    }
    /// Set a stepped color gradient for value-based coloring.
    pub fn gradient(mut self, gradient: ColorGradient) -> Self {
        self.gradient = Some(gradient);
        self
    }
    fn style_for_value(&self, value: f32) -> Style {
        if let Some(gradient) = &self.gradient {
            if let Some(color) = gradient.color_for(value, self.value_range) {
                return self.style.fg(color);
            }
        }
        self.style
    }
    /// Map a value to a Y coordinate within the area
    fn value_to_y(&self, value: f32, height: u16) -> u16 {
        let (min, max) = self.value_range;
        let range = max - min;
        if range.abs() < 0.0001 {
            return height / 2;
        }
        let normalized = (value - min) / range;
        let clamped = normalized.clamp(0.0, 1.0);
        // Invert because Y=0 is at top
        ((1.0 - clamped) * (height - 1) as f32) as u16
    }
    /// Map an X coordinate to a time value (f64 precision)
    fn x_to_time(&self, x: u16, width: u16) -> f64 {
        let (start, end) = self.time_range;
        let normalized = x as f64 / (width - 1).max(1) as f64;
        start + normalized * (end - start)
    }
    /// Map a value to a vertical level (0-4) for a given cell row
    /// Returns the fill level within this cell
    fn value_to_level(&self, value: f32, height: u16, cell_y: u16) -> usize {
        let (min, max) = self.value_range;
        let range = max - min;
        if range.abs() < 0.0001 {
            return 2; // Middle level
        }
        // Normalize value to 0.0-1.0
        let normalized = ((value - min) / range).clamp(0.0, 1.0);
        // Convert to pixel height (inverted: 1.0 = top, 0.0 = bottom)
        let pixel_value = normalized * 100.0;
        // Calculate this cell's vertical range (0-100)
        let cell_top = if height > 1 {
            100.0 * (height - cell_y) as f32 / height as f32
        } else {
            100.0
        };
        let cell_bottom = if height > 1 {
            100.0 * (height - cell_y - 1) as f32 / height as f32
        } else {
            0.0
        };
        // Map value to 0-4 level within this cell
        if pixel_value >= cell_top {
            4 // Fully filled
        } else if pixel_value <= cell_bottom {
            0 // Empty
        } else {
            // Interpolate within cell: map [cell_bottom, cell_top] to [0, 4]
            let cell_range = cell_top - cell_bottom;
            if cell_range.abs() < 0.0001 {
                return 2;
            }
            let position = (pixel_value - cell_bottom) / cell_range;
            ((position * 4.0).round() as usize).clamp(0, 4)
        }
    }
}
impl Widget for SignalView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        match self.render_mode {
            RenderMode::Braille => self.render_braille(area, buf),
            RenderMode::Block => self.render_block(area, buf),
        }
    }
}
impl SignalView<'_> {
    fn render_block(&self, area: Rect, buf: &mut Buffer) {
        // Draw zero line if enabled
        if self.show_zero_line {
            let zero_y = self.value_to_y(0.0, area.height);
            if zero_y < area.height {
                for x in 0..area.width {
                    let cell = buf.cell_mut((area.x + x, area.y + zero_y));
                    if let Some(cell) = cell {
                        cell.set_char('─');
                        cell.set_style(self.zero_line_style);
                    }
                }
            }
        }
        // Draw waveform
        let mut prev_y: Option<u16> = None;
        for x in 0..area.width {
            let t = self.x_to_time(x, area.width);
            let value = self.signal.sample(t);
            let y = self.value_to_y(value, area.height);
            // Draw vertical line from previous Y to current Y for continuity
            if let Some(py) = prev_y {
                let (y_start, y_end) = if py < y { (py, y) } else { (y, py) };
                for draw_y in y_start..=y_end {
                    if draw_y < area.height {
                        let cell = buf.cell_mut((area.x + x, area.y + draw_y));
                        if let Some(cell) = cell {
                            cell.set_char(self.wave_char);
                            cell.set_style(self.style_for_value(value));
                        }
                    }
                }
            } else if y < area.height {
                let cell = buf.cell_mut((area.x + x, area.y + y));
                if let Some(cell) = cell {
                    cell.set_char(self.wave_char);
                    cell.set_style(self.style_for_value(value));
                }
            }
            prev_y = Some(y);
        }
    }
    fn render_braille(&self, area: Rect, buf: &mut Buffer) {
        // btop-style filled area graph using Braille characters
        // Each character encodes transition from previous to current value
        let (time_start, time_end) = self.time_range;
        let time_range = time_end - time_start;
        let symbols = &BRAILLE_GRAPH_SYMBOLS;
        // Sample signal values for each horizontal position
        let values: Vec<f32> = (0..area.width)
            .map(|x| {
                let t = time_start + (x as f64 / area.width.max(1) as f64) * time_range;
                self.signal.sample(t)
            })
            .collect();
        // Render each cell
        for cell_y in 0..area.height {
            let mut prev_level: Option<usize> = None;
            for cell_x in 0..area.width {
                let current_value = values[cell_x as usize];
                let current_level = self.value_to_level(current_value, area.height, cell_y);
                // Determine previous level (from left neighbor or assume 0)
                let prev = prev_level.unwrap_or(current_level);
                // Look up the Braille character from the 5×5 table
                let ch = symbols[prev][current_level];
                // Render the character
                if ch != ' ' {
                    let cell = buf.cell_mut((area.x + cell_x, area.y + cell_y));
                    if let Some(cell) = cell {
                        cell.set_char(ch);
                        cell.set_style(self.style_for_value(current_value));
                    }
                }
                prev_level = Some(current_level);
            }
        }
        // Draw zero line if enabled (as an overlay)
        if self.show_zero_line {
            for cell_x in 0..area.width {
                // Find which cell row contains the zero line
                for cell_y in 0..area.height {
                    let cell_level = self.value_to_level(0.0, area.height, cell_y);
                    if cell_level > 0 && cell_level < 4 {
                        // Zero line crosses this cell - draw a horizontal line
                        let cell = buf.cell_mut((area.x + cell_x, area.y + cell_y));
                        if let Some(cell) = cell {
                            // Overlay a subtle zero line marker
                            if cell.symbol() == " " {
                                cell.set_char('─');
                                cell.set_style(self.zero_line_style);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Constant, Sine};
    #[test]
    fn test_signal_view_creation() {
        let sine = Sine::default();
        let view = SignalView::new(&sine)
            .time_range(0.0, 2.0)
            .value_range(-1.5, 1.5);
        assert_eq!(view.time_range, (0.0, 2.0));
        assert_eq!(view.value_range, (-1.5, 1.5));
    }
    #[test]
    fn test_value_to_y() {
        let const_sig = Constant::new(0.0);
        let view = SignalView::new(&const_sig).value_range(0.0, 1.0);
        // 0 should map to bottom (height - 1)
        let y = view.value_to_y(0.0, 10);
        assert_eq!(y, 9);
        // 1 should map to top (0)
        let y = view.value_to_y(1.0, 10);
        assert_eq!(y, 0);
        // 0.5 should map to middle (height/2)
        let y = view.value_to_y(0.5, 10);
        assert_eq!(y, 4); // (1 - 0.5) * 9 = 4.5 -> 4
    }
    #[test]
    fn test_x_to_time() {
        let const_sig = Constant::new(0.0);
        let view = SignalView::new(&const_sig).time_range(0.0, 1.0);
        assert!((view.x_to_time(0, 10) - 0.0).abs() < 0.01);
        assert!((view.x_to_time(9, 10) - 1.0).abs() < 0.01);
        assert!((view.x_to_time(4, 10) - 0.444).abs() < 0.05);
    }
    #[test]
    fn test_value_to_level_single_row() {
        let const_sig = Constant::new(0.0);
        let view = SignalView::new(&const_sig).value_range(0.0, 1.0);
        let level = view.value_to_level(0.5, 1, 0);
        assert!(level <= 4);
    }
    #[test]
    fn test_color_gradient_steps() {
        let gradient = ColorGradient::new()
            .add_stop(0.0, Color::Blue)
            .add_stop(0.5, Color::Green)
            .add_stop(1.0, Color::Red);
        assert_eq!(gradient.color_for(0.0, (0.0, 1.0)).unwrap(), Color::Blue);
        assert_eq!(gradient.color_for(0.5, (0.0, 1.0)).unwrap(), Color::Green);
        assert_eq!(gradient.color_for(1.0, (0.0, 1.0)).unwrap(), Color::Red);
    }
}

// <FILE>src/visualization/cls_signal_view.rs</FILE> - <DESC>Signal oscilloscope widget</DESC>
// <VERS>END OF VERSION: 1.3.0 - 2025-12-27T13:14:12Z</VERS>
