///
/// A status-line widget that can stack up indicators
/// on the left and right end.
///
/// If you use the constants SLANT_TL_BR and SLANT_BL_TR as
/// separator you can do neo-vim a neovim style statusline.
///
/// ```
///
/// # use ratatui::buffer::Buffer;
/// # use ratatui::prelude::Rect;
/// # use ratatui::style::{Style, Stylize};
/// # use ratatui::text::Span;
/// # use ratatui::widgets::Widget;
/// use rat_widget::statusline_stacked::StatusLineStacked;
///
/// # let area = Rect::default();
/// # let mut buf = Buffer::default();
/// # let buf = &mut buf;
///
/// StatusLineStacked::new()
///     .start("first", "|")
///     .start("second", "|")
///     .end(Span::from("last").on_blue(), "|".on_blue())
///     .center("center message")
///     .render(area, buf);
///
/// ```
///
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use std::marker::PhantomData;

/// Block cut at the diagonal.
pub const SLANT_TL_BR: &str = "\u{e0b8}";
/// Block cut at the diagonal.
pub const SLANT_BL_TR: &str = "\u{e0ba}";

/// Statusline with indicators on the left and right side.
#[derive(Debug, Default, Clone)]
pub struct StatusLineStacked<'a> {
    style: Style,
    left: Vec<(Span<'a>, Span<'a>)>,
    center: Line<'a>,
    right: Vec<(Span<'a>, Span<'a>)>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> StatusLineStacked<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Baseline style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Add to the start group of status flags.
    /// These stack from left to right.
    pub fn start(mut self, text: impl Into<Span<'a>>, gap: impl Into<Span<'a>>) -> Self {
        self.left.push((text.into(), gap.into()));
        self
    }

    /// Centered text.
    pub fn center(mut self, text: impl Into<Line<'a>>) -> Self {
        self.center = text.into();
        self
    }

    /// Add to the end group of status flags.
    /// These stack from right to left.
    pub fn end(mut self, text: impl Into<Span<'a>>, gap: impl Into<Span<'a>>) -> Self {
        self.right.push((text.into(), gap.into()));
        self
    }
}

impl<'a> Widget for StatusLineStacked<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut x_end = area.right();
        for (status, gap) in self.right.iter() {
            let width = status.width() as u16;
            status.render(
                Rect::new(x_end.saturating_sub(width), area.y, width, 1),
                buf,
            );
            x_end = x_end.saturating_sub(width);

            let width = gap.width() as u16;
            gap.render(
                Rect::new(x_end.saturating_sub(width), area.y, width, 1),
                buf,
            );
            x_end = x_end.saturating_sub(width);
        }

        let mut x_start = area.x;
        for (status, gap) in self.left.iter() {
            let width = status.width() as u16;
            status.render(Rect::new(x_start, area.y, width, 1), buf);
            x_start += width;

            let width = gap.width() as u16;
            gap.render(Rect::new(x_start, area.y, width, 1), buf);
            x_start += width;
        }

        self.center.render(
            Rect::new(x_start, area.y, x_end.saturating_sub(x_start), 1),
            buf,
        );
    }
}
