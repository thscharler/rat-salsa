//!
//! Extensions for ratatui Paragraph.
//!

use crate::_private::NonExhaustive;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, Widget, Wrap};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};

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

        let scroll = ScrollArea::new()
            .block(self.block.clone())
            .h_scroll(self.hscroll.clone())
            .v_scroll(self.vscroll.clone());

        state.inner = scroll.inner(
            area,
            ScrollAreaState {
                area,
                h_scroll: Some(&mut state.hscroll),
                v_scroll: Some(&mut state.vscroll),
            },
        );

        state.hscroll.set_max_offset(if self.is_wrap {
            0
        } else {
            let w = self.w.line_width();
            let d = state.inner.width as usize;
            w.saturating_sub(d)
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

        let scroll = ScrollArea::new()
            .block(self.block)
            .h_scroll(self.hscroll)
            .v_scroll(self.vscroll);
        scroll.render(
            area,
            buf,
            &mut ScrollAreaState {
                area,
                h_scroll: Some(&mut state.hscroll),
                v_scroll: Some(&mut state.vscroll),
            },
        );

        self.w
            .scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16))
            .render(state.inner, buf);
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Paragraph<'a> {
    type State = ParagraphState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.layout(area, state);

        self.block.render_ref(area, buf);
        if let Some(vscroll) = &self.vscroll {
            vscroll.render_ref(state.vscroll.area, buf, &mut state.vscroll);
        }
        if let Some(hscroll) = &self.hscroll {
            hscroll.render_ref(state.hscroll.area, buf, &mut state.hscroll);
        }

        self.w
            .clone()
            .scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16))
            .render(state.inner, buf);
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
        let mut sas = ScrollAreaState {
            area: self.inner,
            h_scroll: Some(&mut self.hscroll),
            v_scroll: Some(&mut self.vscroll),
        };
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
