//!
//! Extensions for ratatui Paragraph.
//!
use crate::_private::NonExhaustive;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{layout_scroll, Scroll, ScrollArea, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{StatefulWidget, Style, Text, Widget};
use ratatui::widgets::{Block, StatefulWidgetRef, WidgetRef, Wrap};

/// List widget.
///
/// Fully compatible with ratatui Paragraph.
/// Add Scroll and event-handling.
#[derive(Debug, Default)]
pub struct Paragraph<'a> {
    w: ratatui::widgets::Paragraph<'a>,
    is_wrap: bool,
    block: Option<Block<'a>>,
    vscroll: Option<Scroll<'a>>,
    hscroll: Option<Scroll<'a>>,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct ParagraphState {
    /// Full area of the widget.
    pub area: Rect,
    /// Inner area of the widget.
    pub inner: Rect,

    /// Focus
    pub focus: FocusFlag,

    /// Vertical scroll
    pub vscroll: ScrollState,
    /// Horizontal scroll
    pub hscroll: ScrollState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Paragraph<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        let t = text.into();
        Self {
            w: ratatui::widgets::Paragraph::new(t),
            ..Self::default()
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

    /// Base style.
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.w = self.w.style(style);
        self
    }

    /// Word wrap.
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.is_wrap = true;
        self.w = self.w.wrap(wrap);
        self
    }

    /// Text alignment.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.w = self.w.alignment(alignment);
        self
    }

    /// Text alignment.
    pub fn left_aligned(mut self) -> Self {
        self.w = self.w.left_aligned();
        self
    }

    /// Text alignment.
    pub fn centered(mut self) -> Self {
        self.w = self.w.centered();
        self
    }

    /// Text alignment.
    pub fn right_aligned(mut self) -> Self {
        self.w = self.w.right_aligned();
        self
    }
}

impl<'a> Paragraph<'a> {
    fn layout(&self, area: Rect, state: &mut ParagraphState) {
        state.area = area;

        (state.hscroll.area, state.vscroll.area, state.inner) = layout_scroll(
            area,
            self.block.as_ref(),
            self.hscroll.as_ref(),
            self.vscroll.as_ref(),
        );

        state.hscroll.set_max_offset(if self.is_wrap {
            0
        } else {
            self.w
                .line_width()
                .saturating_sub(state.inner.width as usize)
        });
        state.hscroll.set_page_len(state.inner.width as usize);

        // 4 is an estimate. line_count seems not very accurate.
        let lines = self.w.line_count(area.width) + 4;
        state
            .vscroll
            .set_max_offset(lines.saturating_sub(state.inner.height as usize));
        state.vscroll.set_page_len(state.inner.height as usize);
    }
}

impl<'a> StatefulWidget for Paragraph<'a> {
    type State = ParagraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.layout(area, state);
        render_para(&self, area, buf, state);

        self.w
            .scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16))
            .render(state.inner, buf);
    }
}

impl<'a> StatefulWidgetRef for Paragraph<'a> {
    type State = ParagraphState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.layout(area, state);
        render_para(self, area, buf, state);

        self.w
            .clone()
            .scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16))
            .render(state.inner, buf);
    }
}

fn render_para(widget: &Paragraph<'_>, area: Rect, buf: &mut Buffer, state: &mut ParagraphState) {
    widget.block.render_ref(area, buf);
    if let Some(vscroll) = &widget.vscroll {
        vscroll.render_ref(state.vscroll.area, buf, &mut state.vscroll);
    }
    if let Some(hscroll) = &widget.hscroll {
        hscroll.render_ref(state.hscroll.area, buf, &mut state.hscroll);
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

impl HasFocusFlag for ParagraphState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl ParagraphState {
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
                ct_event!(keycode press Down) => self.scroll_up(1).into(),
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
        match ScrollArea(self.inner, Some(&mut self.hscroll), Some(&mut self.vscroll))
            .handle(event, MouseOnly)
        {
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
