//!
//! Line numbers widget.
//!

use crate::_private::NonExhaustive;
use crate::text_area::TextAreaState;
use crate::{TextPosition, upos_type};
use format_num_pattern::NumberFormat;
use rat_event::util::MouseFlags;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::{Block, Widget};

/// Renders line-numbers.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`LineNumberState`] to handle common actions.
#[derive(Debug, Default, Clone)]
pub struct LineNumbers<'a> {
    start: Option<upos_type>,
    end: Option<upos_type>,
    cursor: Option<upos_type>,
    text_area: Option<&'a TextAreaState>,

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

    /// First rendered line-number
    pub start: upos_type,

    /// Helper for mouse.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> LineNumbers<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sync with this text-area.
    ///
    /// To make this work correctly, the TextArea must be rendered
    /// first to make sure that all layout-information stored in the
    /// state is accurate.
    pub fn with_textarea(mut self, text_area: &'a TextAreaState) -> Self {
        self.text_area = Some(text_area);
        self
    }

    /// Start position.
    pub fn start(mut self, start: upos_type) -> Self {
        self.start = Some(start);
        self
    }

    /// End position.
    pub fn end(mut self, end: upos_type) -> Self {
        self.end = Some(end);
        self
    }

    /// Current line for highlighting.
    pub fn cursor(mut self, cursor: upos_type) -> Self {
        self.cursor = Some(cursor);
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

    /// Extra margin as (left-margin, right-margin).
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
    #[deprecated(since = "1.1.0", note = "use width_for()")]
    pub fn width(&self) -> u16 {
        let nr_width = if let Some(text_area) = self.text_area {
            (text_area.vscroll.offset() + 50).ilog10() as u16 + 1
        } else if let Some(end) = self.end {
            end.ilog10() as u16 + 1
        } else if let Some(start) = self.start {
            (start + 50).ilog10() as u16 + 1
        } else {
            3
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

    /// Required width for the line-numbers.
    pub fn width_for(start_nr: usize, flag_width: u16, margin: (u16, u16), block: u16) -> u16 {
        let nr_width = (start_nr + 50).ilog10() as u16 + 1;
        nr_width + flag_width + margin.0 + margin.1 + block + 1
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

    #[allow(clippy::manual_unwrap_or_default)]
    #[allow(clippy::manual_unwrap_or)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        state.start = if let Some(text_area) = self.text_area {
            text_area.offset().1 as upos_type
        } else if let Some(start) = self.start {
            start
        } else {
            0
        };
        let end = if let Some(text_area) = self.text_area {
            text_area.len_lines()
        } else if let Some(end) = self.end {
            end
        } else {
            state.start + state.inner.height as upos_type
        };

        let nr_width = if let Some(text_area) = self.text_area {
            (text_area.vscroll.offset() + 50).ilog10() as u16 + 1
        } else if let Some(end) = self.end {
            end.ilog10() as u16 + 1
        } else if let Some(start) = self.start {
            (start + 50).ilog10() as u16 + 1
        } else {
            3
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

        let cursor = if let Some(text_area) = self.text_area {
            text_area.cursor()
        } else if let Some(cursor) = self.cursor {
            TextPosition::new(0, cursor)
        } else {
            TextPosition::new(0, upos_type::MAX)
        };

        let mut tmp = String::new();
        let mut prev_nr = upos_type::MAX;

        for y in state.inner.top()..state.inner.bottom() {
            let nr;
            let rel_nr;
            let render_nr;
            let render_cursor;

            if let Some(text_area) = self.text_area {
                let rel_y = y - state.inner.y;
                if let Some(pos) = text_area.relative_screen_to_pos((0, rel_y as i16)) {
                    nr = pos.y;
                    if self.relative {
                        rel_nr = nr.abs_diff(cursor.y);
                    } else {
                        rel_nr = nr;
                    }
                    render_nr = pos.y != prev_nr;
                    render_cursor = pos.y == cursor.y;
                } else {
                    nr = 0;
                    rel_nr = 0;
                    render_nr = false;
                    render_cursor = false;
                }
            } else {
                nr = state.start + (y - state.inner.y) as upos_type;
                render_nr = nr < end;
                render_cursor = Some(nr) == self.cursor;
                if self.relative {
                    rel_nr = nr.abs_diff(self.cursor.unwrap_or_default());
                } else {
                    rel_nr = nr;
                }
            }

            tmp.clear();
            if render_nr {
                _ = format.fmt_to(rel_nr, &mut tmp);
            }

            let style = if render_cursor {
                cursor_style
            } else {
                self.style
            };

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

            prev_nr = nr;
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
