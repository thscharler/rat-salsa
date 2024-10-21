//!
//! Extensions for ratatui Paragraph.
//!

use crate::_private::NonExhaustive;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocus};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Line;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, Widget, Wrap};

/// List widget.
///
/// Fully compatible with ratatui Paragraph.
/// Add Scroll and event-handling.
#[derive(Debug, Default)]
pub struct Paragraph<'a> {
    text: Text<'a>,
    style: Style,
    wrap: Option<Wrap>,
    alignment: Option<Alignment>,
    block: Option<Block<'a>>,
    vscroll: Option<Scroll<'a>>,
    hscroll: Option<Scroll<'a>>,
}

#[derive(Debug)]
pub struct ParagraphStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,
    pub non_exhaustive: NonExhaustive,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct ParagraphState {
    /// Full area of the widget.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Inner area of the widget.
    /// __readonly__. renewed for each render.
    pub inner: Rect,

    /// Vertical scroll.
    /// __read+write__
    pub vscroll: ScrollState,
    /// Horizontal scroll.
    /// __read+write__
    pub hscroll: ScrollState,

    /// Focus.
    /// __read+write__
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ParagraphStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Paragraph<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set both hscroll and vscroll.
    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.clone().override_horizontal());
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Set horizontal scroll.
    pub fn hscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.override_horizontal());
        self
    }

    /// Set vertical scroll.
    pub fn vscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Styles.
    pub fn styles(mut self, styles: ParagraphStyle) -> Self {
        self.style = styles.style;
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles));
        }
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Word wrap.
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Text alignment.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Text alignment.
    pub fn left_aligned(mut self) -> Self {
        self.alignment = Some(Alignment::Left);
        self
    }

    /// Text alignment.
    pub fn centered(mut self) -> Self {
        self.alignment = Some(Alignment::Center);
        self
    }

    /// Text alignment.
    pub fn right_aligned(mut self) -> Self {
        self.alignment = Some(Alignment::Right);
        self
    }
}

impl<'a> StatefulWidget for Paragraph<'a> {
    type State = ParagraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());

        state.inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        state.hscroll.set_max_offset(if self.wrap.is_some() {
            0
        } else {
            let max = self.text.iter().map(Line::width).max().unwrap_or_default();
            max.saturating_sub(state.inner.width as usize)
        });
        state.hscroll.set_page_len(state.inner.width as usize);

        // paragraph
        let mut para = ratatui::widgets::Paragraph::new(self.text)
            .style(self.style)
            .scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16));
        if let Some(alignment) = self.alignment {
            para = para.alignment(alignment);
        }
        if let Some(wrap) = self.wrap {
            para = para.wrap(wrap);
        }
        let lines = para.line_count(state.inner.width) + 4;
        para.render(state.inner, buf);

        state
            .vscroll
            .set_max_offset(lines.saturating_sub(state.inner.height as usize));
        state.vscroll.set_page_len(state.inner.height as usize);

        sa.render(
            area,
            buf,
            &mut ScrollAreaState::new()
                .area(area)
                .h_scroll(&mut state.hscroll)
                .v_scroll(&mut state.vscroll),
        );
    }
}

impl HasFocus for ParagraphState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl Default for ParagraphState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            focus: Default::default(),
            vscroll: Default::default(),
            hscroll: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ParagraphState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Self::default()
        }
    }

    /// Current offset.
    pub fn line_offset(&self) -> usize {
        self.vscroll.offset()
    }

    /// Set limited offset.
    pub fn set_line_offset(&mut self, offset: usize) -> bool {
        self.vscroll.set_offset(offset)
    }

    /// Current offset.
    pub fn col_offset(&self) -> usize {
        self.hscroll.offset()
    }

    /// Set limited offset.
    pub fn set_col_offset(&mut self, offset: usize) -> bool {
        self.hscroll.set_offset(offset)
    }

    /// Scroll left by n.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.hscroll.scroll_left(n)
    }

    /// Scroll right by n.
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.hscroll.scroll_right(n)
    }

    /// Scroll up by n.
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.vscroll.scroll_up(n)
    }

    /// Scroll down by n.
    pub fn scroll_down(&mut self, n: usize) -> bool {
        self.vscroll.scroll_down(n)
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ParagraphState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        flow!(if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.scroll_up(1).into(),
                ct_event!(keycode press Down) => self.scroll_down(1).into(),
                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press PageUp) => self.scroll_up(self.vscroll.page_len()).into(),
                ct_event!(keycode press PageDown) => self.scroll_down(self.vscroll.page_len).into(),
                ct_event!(keycode press Home) => self.set_line_offset(0).into(),
                ct_event!(keycode press End) => self.set_line_offset(1).into(),
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        });

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ParagraphState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        let mut sas = ScrollAreaState::new()
            .area(self.inner)
            .h_scroll(&mut self.hscroll)
            .v_scroll(&mut self.vscroll);
        match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(v) => {
                if self.scroll_up(v) {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                }
            }
            ScrollOutcome::Down(v) => {
                if self.scroll_down(v) {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                }
            }
            ScrollOutcome::Left(v) => {
                if self.scroll_left(v) {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                }
            }
            ScrollOutcome::Right(v) => {
                if self.scroll_right(v) {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                }
            }
            ScrollOutcome::VPos(v) => self.set_line_offset(v).into(),
            ScrollOutcome::HPos(v) => self.set_col_offset(v).into(),
            r => r.into(),
        }
    }
}
