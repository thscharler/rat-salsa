//!
//! Line numbers widget.
//!

use crate::_private::NonExhaustive;
use crate::upos_type;
use format_num_pattern::NumberFormat;
use rat_event::util::MouseFlags;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::text::Line;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_widgets::block::{Block, BlockExt};

/// Renders line-numbers.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`LineNumberState`] to handle common actions.
#[derive(Debug, Default, Clone)]
pub struct LineNumbers<'a> {
    start: upos_type,
    end: Option<upos_type>,
    cursor: upos_type,
    relative: bool,
    flags: Vec<Line<'a>>,

    flag_width: Option<u16>,
    margin: (u16, u16),

    format: Option<NumberFormat>,
    style: Style,
    cursor_style: Option<Style>,

    block: Option<Block<'a>>,
}

/// Styles as a package.
#[derive(Debug, Clone)]
pub struct LineNumberStyle {
    pub flag_width: Option<u16>,
    pub margin: Option<(u16, u16)>,
    pub format: Option<NumberFormat>,
    pub style: Style,
    pub cursor: Option<Style>,
    pub block: Option<Block<'static>>,

    pub non_exhaustive: NonExhaustive,
}

/// State
#[derive(Debug, Clone)]
pub struct LineNumberState {
    pub area: Rect,
    pub inner: Rect,

    pub start: upos_type,

    /// Helper for mouse.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> LineNumbers<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start position.
    pub fn start(mut self, start: upos_type) -> Self {
        self.start = start;
        self
    }

    /// End position.
    pub fn end(mut self, end: upos_type) -> Self {
        self.end = Some(end);
        self
    }

    /// Current line for highlighting.
    pub fn cursor(mut self, cursor: upos_type) -> Self {
        self.cursor = cursor;
        self
    }

    /// Numbering relative to cursor
    pub fn relative(mut self, relative: bool) -> Self {
        self.relative = relative;
        self
    }

    /// Extra info.
    pub fn flags(mut self, flags: Vec<Line<'a>>) -> Self {
        self.flags = flags;
        self
    }

    /// Required width for the flags.
    pub fn flag_width(mut self, width: u16) -> Self {
        self.flag_width = Some(width);
        self
    }

    /// Extra margin.
    pub fn margin(mut self, margin: (u16, u16)) -> Self {
        self.margin = margin;
        self
    }

    /// Line number format.
    pub fn format(mut self, format: NumberFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Complete set of styles.
    pub fn styles(mut self, styles: LineNumberStyle) -> Self {
        self.style = styles.style;
        if let Some(flag_width) = styles.flag_width {
            self.flag_width = Some(flag_width);
        }
        if let Some(margin) = styles.margin {
            self.margin = margin;
        }
        if let Some(format) = styles.format {
            self.format = Some(format);
        }
        if let Some(cursor_style) = styles.cursor {
            self.cursor_style = Some(cursor_style);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self
    }

    /// Style for current line.
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = Some(style);
        self
    }

    /// Block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block.style(self.style));
        self
    }

    /// Calculates the necessary width for the configuration.
    pub fn width(&self) -> u16 {
        let nr_width = if let Some(end) = self.end {
            end.ilog10() as u16 + 1
        } else {
            (self.start + 100).ilog10() as u16 + 1
        };
        let flag_width = if let Some(flag_width) = self.flag_width {
            flag_width
        } else {
            self.flags
                .iter()
                .map(|v| v.width() as u16)
                .max()
                .unwrap_or_default()
        };
        let block_width = {
            let area = self.block.inner_if_some(Rect::new(0, 0, 2, 2));
            2 - area.width
        };

        nr_width + flag_width + self.margin.0 + self.margin.1 + block_width + 1
    }
}

impl Default for LineNumberStyle {
    fn default() -> Self {
        Self {
            flag_width: None,
            margin: None,
            format: None,
            style: Default::default(),
            cursor: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl StatefulWidget for LineNumbers<'_> {
    type State = LineNumberState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);
        state.start = self.start;
        let end = self.end.unwrap_or(upos_type::MAX);

        let nr_width = if let Some(end) = self.end {
            end.ilog10() as u16 + 1
        } else {
            (self.start + 100).ilog10() as u16 + 1
        };

        let flag_width = if let Some(flag_width) = self.flag_width {
            flag_width
        } else {
            self.flags
                .iter()
                .map(|v| v.width() as u16)
                .max()
                .unwrap_or_default()
        };

        let format = if let Some(format) = self.format {
            format
        } else {
            let mut f = "#".repeat(nr_width.saturating_sub(1) as usize);
            f.push('0');
            NumberFormat::new(f).expect("valid")
        };

        let cursor_style = if let Some(cursor_style) = self.cursor_style {
            cursor_style
        } else {
            self.style
        };

        if let Some(block) = self.block {
            block.render(area, buf);
        } else {
            buf.set_style(area, self.style);
        }

        let mut tmp = String::new();
        for y in state.inner.top()..state.inner.bottom() {
            let (nr, is_cursor) = if self.relative {
                let pos = self.start + (y - state.inner.y) as upos_type;
                (pos.abs_diff(self.cursor), pos == self.cursor)
            } else {
                let pos = self.start + (y - state.inner.y) as upos_type;
                (pos, pos == self.cursor)
            };

            tmp.clear();
            if nr < end {
                _ = format.fmt_to(nr, &mut tmp);
            }

            let style = if is_cursor { cursor_style } else { self.style };

            let nr_area = Rect::new(
                state.inner.x + self.margin.0, //
                y,
                nr_width,
                1,
            )
            .intersection(area);
            buf.set_stringn(nr_area.x, nr_area.y, &tmp, nr_area.width as usize, style);

            if let Some(flags) = self.flags.get((y - state.inner.y) as usize) {
                flags.render(
                    Rect::new(
                        state.inner.x + self.margin.0 + nr_width + 1,
                        y,
                        flag_width,
                        1,
                    ),
                    buf,
                );
            }
        }
    }
}

impl Default for LineNumberState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            start: 0,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl LineNumberState {
    pub fn new() -> Self {
        Self::default()
    }
}
