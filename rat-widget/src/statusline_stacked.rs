//! A status-line widget that can stack up indicators
//! on the left and right end.
//!
//! If you use the constants SLANT_TL_BR and SLANT_BL_TR as
//! separator you can do neo-vim a neovim style statusline.
//!
//! ```
//!
//! # use std::time::Duration;
//! use ratatui_core::buffer::Buffer;
//! # use ratatui_core::layout::Rect;
//! # use ratatui_core::style::{Color, Style, Stylize};
//! # use ratatui_core::text::Span;
//! # use ratatui_core::widgets::Widget;
//! use rat_widget::statusline_stacked::{StatusLineStacked, SLANT_BL_TR, SLANT_TL_BR};
//!
//! # let area = Rect::default();
//! # let mut buf = Buffer::default();
//! # let buf = &mut buf;
//!
//! let color_0 = Color::DarkGray;
//! let color_1 = Color::Green;
//! let color_3 = Color::Cyan;
//! let color_4 = Color::DarkGray;
//!
//! StatusLineStacked::new()
//!     .start(
//!         Span::from(" STATUS-0 ")
//!             .style(Style::new().fg(Color::Black).bg(color_0)),
//!         Span::from(SLANT_TL_BR).style(Style::new().fg(color_0).bg(color_1)),
//!     )
//!     .start(
//!         Span::from(" STATUS-1 ").style(Style::new().fg(Color::Black).bg(color_1)),
//!         Span::from(SLANT_TL_BR).style(Style::new().fg(color_1)),
//!     )
//!     .center_margin(1)
//!     .center("Some status message ...")
//!     .end(
//!         Span::from(format!("R[{:.0?} ", Duration::from_micros(25)))
//!             .style(Style::new().fg(Color::Black).bg(color_3)),
//!         Span::from(SLANT_BL_TR).style(Style::new().fg(color_3).bg(color_4)),
//!     )
//!     .end(
//!         "",
//!         Span::from(SLANT_BL_TR).style(Style::new().fg(color_4).bg(color_3)),
//!     )
//!     .end(
//!         Span::from(format!("E[{:.0?}", Duration::from_micros(17)))
//!             .style(Style::new().fg(Color::Black).bg(color_3)),
//!         Span::from(SLANT_BL_TR).style(Style::new().fg(color_3).bg(color_4)),
//!     )
//!     .end(
//!         "",
//!         Span::from(SLANT_BL_TR).style(Style::new().fg(color_4).bg(color_3)),
//!     )
//!     .end("", Span::from(SLANT_BL_TR).style(Style::new().fg(color_4)))
//!     .render(area, buf);
//!
//! ```
//!
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::text::Line;
use ratatui_core::widgets::Widget;
use std::marker::PhantomData;

/// Block cut at the diagonal.
pub const SLANT_TL_BR: &str = "\u{e0b8}";
/// Block cut at the diagonal.
pub const SLANT_BL_TR: &str = "\u{e0ba}";

/// Statusline with indicators on the left and right side.
#[derive(Debug, Default, Clone)]
pub struct StatusLineStacked<'a> {
    style: Style,
    left: Vec<(Line<'a>, Line<'a>)>,
    center_margin: u16,
    center: Line<'a>,
    right: Vec<(Line<'a>, Line<'a>)>,
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
    pub fn start(mut self, text: impl Into<Line<'a>>, gap: impl Into<Line<'a>>) -> Self {
        self.left.push((text.into(), gap.into()));
        self
    }

    /// Add to the start group of status flags.
    /// These stack from left to right.
    pub fn start_bare(mut self, text: impl Into<Line<'a>>) -> Self {
        self.left.push((text.into(), "".into()));
        self
    }

    /// Margin around centered text.
    pub fn center_margin(mut self, margin: u16) -> Self {
        self.center_margin = margin;
        self
    }

    /// Centered text.
    pub fn center(mut self, text: impl Into<Line<'a>>) -> Self {
        self.center = text.into();
        self
    }

    /// Add to the end group of status flags.
    /// These stack from right to left.
    pub fn end(mut self, text: impl Into<Line<'a>>, gap: impl Into<Line<'a>>) -> Self {
        self.right.push((text.into(), gap.into()));
        self
    }

    /// Add to the end group of status flags.
    /// These stack from right to left.
    pub fn end_bare(mut self, text: impl Into<Line<'a>>) -> Self {
        self.right.push((text.into(), "".into()));
        self
    }
}

impl<'a> Widget for StatusLineStacked<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut x_end = area.right();
        for (status, gap) in self.right {
            let width = status.width() as u16;
            status.style(self.style).render(
                Rect::new(x_end.saturating_sub(width), area.y, width, 1),
                buf,
            );
            x_end = x_end.saturating_sub(width);

            let width = gap.width() as u16;
            gap.style(self.style).render(
                Rect::new(x_end.saturating_sub(width), area.y, width, 1),
                buf,
            );
            x_end = x_end.saturating_sub(width);
        }

        let mut x_start = area.x;
        for (status, gap) in self.left {
            let width = status.width() as u16;
            status
                .style(self.style)
                .render(Rect::new(x_start, area.y, width, 1), buf);
            x_start += width;

            let width = gap.width() as u16;
            gap.style(self.style)
                .render(Rect::new(x_start, area.y, width, 1), buf);
            x_start += width;
        }

        // middle area
        buf.set_style(
            Rect::new(x_start, area.y, x_end.saturating_sub(x_start), 1),
            self.style,
        );

        let center_width = x_end
            .saturating_sub(x_start)
            .saturating_sub(self.center_margin * 2);
        self.center.style(self.style).render(
            Rect::new(x_start + self.center_margin, area.y, center_width, 1),
            buf,
        );
    }
}
