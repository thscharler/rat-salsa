use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use std::marker::PhantomData;

/// Block cut at the diagonal.
pub const SLANT_TL_BR: char = '\u{e0b8}';
/// Block cut at the diagonal.
pub const SLANT_BL_TR: char = '\u{e0ba}';

#[derive(Debug, Default, Clone)]
pub struct StatusStack<'a> {
    style: Style,
    left: Vec<(Span<'a>, Option<(char, Style)>)>,
    center: Line<'a>,
    right: Vec<(Span<'a>, Option<(char, Style)>)>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> StatusStack<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Add to the start group of status flags.
    /// These stack from left to right.
    pub fn start(mut self, text: impl Into<Span<'a>>, gap: Option<(char, Style)>) -> Self {
        self.left.push((text.into(), gap));
        self
    }

    /// Centered text.
    pub fn center(mut self, text: impl Into<Line<'a>>) -> Self {
        self.center = text.into();
        self
    }

    /// Add to the end group of status flags.
    /// These stack from right to left.
    pub fn end(mut self, text: impl Into<Span<'a>>, gap: Option<(char, Style)>) -> Self {
        self.right.push((text.into(), gap));
        self
    }
}

impl<'a> Widget for StatusStack<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        debug!("RENDER");

        let mut x_end = area.right();
        for (v, g) in self.right.iter() {
            let width = v.width() as u16;

            v.render(
                Rect::new(x_end.saturating_sub(width), area.y, width, 1),
                buf,
            );
            x_end = x_end.saturating_sub(width);
            if let Some((gc, gs)) = g {
                let gc = gc.to_string();
                let gc_width = unicode_display_width::width(&gc) as u16;

                if let Some(cell) =
                    buf.cell_mut(Position::new(x_end.saturating_sub(gc_width), area.y))
                {
                    cell.set_style(*gs);
                    cell.set_symbol(&gc);
                    x_end = x_end.saturating_sub(gc_width);
                }
            }
        }

        let mut x_start = area.x;
        for (v, g) in self.left.iter() {
            let width = v.width() as u16;
            debug!("left {:?} {}", v, width);

            v.render(Rect::new(x_start, area.y, width, 1), buf);
            x_start += width;
            if let Some((gc, gs)) = g {
                let gc = gc.to_string();
                let gc_width = unicode_display_width::width(&gc) as u16;
                debug!("gc {:?} {:?}", gc, gc_width);

                if let Some(cell) = buf.cell_mut(Position::new(x_start, area.y)) {
                    cell.set_style(*gs);
                    cell.set_symbol(&gc);
                    x_start += gc_width;
                }
            }
        }

        self.center.render(
            Rect::new(x_start, area.y, x_end.saturating_sub(x_start), 1),
            buf,
        );
    }
}
