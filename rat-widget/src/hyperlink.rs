//!
//! Render a span as a hyperlink.
//!

use ratatui_core::buffer::{Buffer, CellDiffOption, CellWidth};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Style, Styled};
use ratatui_core::text::Span;
use ratatui_core::widgets::Widget;
use std::num::NonZeroU16;

#[derive(Debug, Default)]
pub struct Hyperlink<'a> {
    span: Span<'a>,
    link: String,
}

impl<'a> Hyperlink<'a> {
    pub fn new(span: impl Into<Span<'a>>, link: impl Into<String>) -> Self {
        Self {
            span: span.into(),
            link: link.into(),
        }
    }

    pub fn span(mut self, span: Span<'a>) -> Self {
        self.span = span;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.span = self.span.style(style.into());
        self
    }

    pub fn patch_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.span = self.span.patch_style(style);
        self
    }

    pub fn reset_style(self) -> Self {
        self.patch_style(Style::reset())
    }

    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link = link.into();
        self
    }

    pub fn width(&self) -> usize {
        self.span.width()
    }
}

impl Styled for Hyperlink<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.span.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> Widget for Hyperlink<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        let Rect { mut x, y, .. } = area;
        let mut link_text = String::new();
        for grapheme in self.span.styled_graphemes(Style::default()) {
            let symbol_width = grapheme.symbol.cell_width();
            let next_x = x.saturating_add(symbol_width);
            if next_x > area.right() {
                break;
            }

            link_text.push_str(grapheme.symbol);

            x = next_x;
        }

        if let Some(cell) = buf.cell_mut(Position::new(area.x, area.y)) {
            cell.set_style(self.span.style);
            cell.set_symbol(
                format!(
                    "\u{1B}]8;;{}\u{1B}\\{}\u{1B}]8;;\u{1B}\\",
                    self.link, link_text
                )
                    .as_str(),
            );
            cell.set_diff_option(CellDiffOption::ForcedWidth(
                NonZeroU16::new(1).expect("fine"),
            ));
        }

        let Rect { mut x, y, .. } = area;
        for grapheme in self.span.styled_graphemes(Style::default()) {
            let symbol_width = grapheme.symbol.cell_width();
            let next_x = x.saturating_add(symbol_width);
            if next_x > area.right() {
                break;
            }

            if x == area.x {
                // don't skip the first cell
            } else {
                for x_hidden in x..next_x {
                    buf[(x_hidden, y)].set_diff_option(CellDiffOption::Skip);
                }
            }

            x = next_x;
        }
    }
}
