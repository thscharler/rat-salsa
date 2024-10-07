//!
//! Line numbers widget.
//!

use crate::_private::NonExhaustive;
use crate::upos_type;
use format_num_pattern::NumberFormat;
use rat_event::util::MouseFlags;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{BlockExt, StatefulWidget, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

/// Renders line-numbers.
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
    pub cursor_style: Option<Style>,
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
        if let Some(flag_width) = styles.flag_width {
            self.flag_width = Some(flag_width);
        }
        if let Some(margin) = styles.margin {
            self.margin = margin;
        }
        if let Some(format) = styles.format {
            self.format = Some(format);
        }
        self.style = styles.style;
        if let Some(cursor_style) = styles.cursor_style {
            self.cursor_style = Some(cursor_style);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for current line.
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = Some(style);
        self
    }

    /// Block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
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
            cursor_style: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> StatefulWidget for LineNumbers<'a> {
    type State = LineNumberState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);
        let inner = state.inner;
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

        self.block.render(area, buf);
        // set base style
        for y in inner.top()..inner.bottom() {
            for x in inner.left()..inner.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    cell.set_style(self.style);
                }
            }
        }

        let mut tmp = String::new();
        for y in inner.top()..inner.bottom() {
            let (nr, is_cursor) = if self.relative {
                let pos = self.start + (y - inner.y) as upos_type;
                (pos.abs_diff(self.cursor), pos == self.cursor)
            } else {
                let pos = self.start + (y - inner.y) as upos_type;
                (pos, pos == self.cursor)
            };

            tmp.clear();
            if nr < end {
                _ = format.fmt_to(nr, &mut tmp);
            }

            if is_cursor {
                for x in inner.x + self.margin.0..inner.x + self.margin.0 + nr_width {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                        cell.set_style(cursor_style);
                    }
                }
            }
            buf.set_string(inner.x + self.margin.0, y, &tmp, Style::default());
            if let Some(flags) = self.flags.get((y - inner.y) as usize) {
                flags.render(
                    Rect::new(inner.x + self.margin.0 + nr_width + 1, y, flag_width, 1),
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
