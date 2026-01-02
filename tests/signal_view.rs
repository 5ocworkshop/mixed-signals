#[cfg(feature = "visualization")]
mod visualization_tests {
    use mixed_signals::prelude::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::Widget;

    #[test]
    fn signal_view_render_block_single_row() {
        let signal = Constant::new(0.0);
        let view = SignalView::new(&signal).render_mode(RenderMode::Block);
        let area = Rect::new(0, 0, 4, 1);
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
    }

    #[test]
    fn signal_view_render_braille_single_row() {
        let signal = Constant::new(0.0);
        let view = SignalView::new(&signal).render_mode(RenderMode::Braille);
        let area = Rect::new(0, 0, 4, 1);
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
    }
}
