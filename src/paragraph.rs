//!
//! Extensions for ratatui Paragraph.
//!

use crate::_private::NonExhaustive;
use crate::util::revert_style;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocus};
use rat_reloc::{relocate_area, RelocatableState};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Position, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget, Wrap};
use std::cell::RefCell;
use std::cmp::min;
use std::mem;
use std::ops::DerefMut;

/// List widget.
///
/// Fully compatible with ratatui Paragraph.
/// Add Scroll and event-handling.
#[derive(Debug, Default)]
pub struct Paragraph<'a> {
    style: Style,
    focus_style: Option<Style>,

    wrap: Option<Wrap>,
    para: RefCell<ratatui::widgets::Paragraph<'a>>,

    block: Option<Block<'a>>,
    vscroll: Option<Scroll<'a>>,
    hscroll: Option<Scroll<'a>>,
}

#[derive(Debug)]
pub struct ParagraphStyle {
    pub style: Style,
    pub focus: Option<Style>,

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

    /// Text lines
    pub lines: usize,

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
            focus: None,
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
            para: RefCell::new(ratatui::widgets::Paragraph::new(text)),
            ..Default::default()
        }
    }

    /// Text
    pub fn text(mut self, text: impl Into<Text<'a>>) -> Self {
        let mut para = ratatui::widgets::Paragraph::new(text);
        if let Some(wrap) = self.wrap {
            para = para.wrap(wrap);
        }
        self.para = RefCell::new(para);
        self
    }

    /// Block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
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
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles));
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Word wrap.
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);

        let mut para = mem::take(self.para.borrow_mut().deref_mut());
        para = para.wrap(wrap);
        self.para = RefCell::new(para);

        self
    }

    /// Text alignment.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        let mut para = mem::take(self.para.borrow_mut().deref_mut());
        para = para.alignment(alignment);
        self.para = RefCell::new(para);

        self
    }

    /// Text alignment.
    pub fn left_aligned(mut self) -> Self {
        self.alignment(Alignment::Left)
    }

    /// Text alignment.
    pub fn centered(mut self) -> Self {
        self.alignment(Alignment::Center)
    }

    /// Text alignment.
    pub fn right_aligned(mut self) -> Self {
        self.alignment(Alignment::Right)
    }

    /// Line width when not wrapped.
    pub fn line_width(&self) -> usize {
        self.para.borrow().line_width()
    }

    /// Line height for the supposed width.
    pub fn line_height(&self, width: u16) -> usize {
        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        let padding = sa.padding();

        self.para
            .borrow()
            .line_count(width.saturating_sub(padding.left + padding.right))
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Paragraph<'a> {
    type State = ParagraphState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_paragraph(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for Paragraph<'a> {
    type State = ParagraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_paragraph(&self, area, buf, state);
    }
}

fn render_paragraph(
    widget: &Paragraph<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ParagraphState,
) {
    state.area = area;

    // take paragraph
    let mut para = mem::take(widget.para.borrow_mut().deref_mut());

    // update scroll
    let sa = ScrollArea::new()
        .block(widget.block.as_ref())
        .h_scroll(widget.hscroll.as_ref())
        .v_scroll(widget.vscroll.as_ref());
    // not the final inner, showing the scrollbar might change this.
    let tmp_inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));
    let pad_inner = sa.padding();

    state.lines = para.line_count(area.width.saturating_sub(pad_inner.left + pad_inner.right));

    state
        .vscroll
        .set_max_offset(state.lines.saturating_sub(tmp_inner.height as usize));
    state.vscroll.set_page_len(tmp_inner.height as usize);
    state.hscroll.set_max_offset(if widget.wrap.is_some() {
        0
    } else {
        para.line_width().saturating_sub(tmp_inner.width as usize)
    });
    state.hscroll.set_page_len(tmp_inner.width as usize);
    state.inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

    sa.render(
        area,
        buf,
        &mut ScrollAreaState::new()
            .h_scroll(&mut state.hscroll)
            .v_scroll(&mut state.vscroll),
    );

    para = para.scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16));
    (&para).render(state.inner, buf);

    if state.is_focused() {
        let focus_style = widget.focus_style.unwrap_or(revert_style(widget.style));

        let mut tag = None;
        for x in state.inner.left()..state.inner.right() {
            if let Some(cell) = buf.cell_mut(Position::new(x, state.inner.y)) {
                if tag.is_none() {
                    if cell.symbol() != " " {
                        tag = Some(true);
                    }
                } else {
                    if cell.symbol() == " " {
                        tag = Some(false);
                    }
                }
                if tag == Some(true) || (x - state.inner.x < 3) {
                    cell.set_style(focus_style);
                }
            }
        }

        let y = min(
            state.inner.y as usize + state.vscroll.page_len() / 2,
            (state.inner.y as usize + state.vscroll.max_offset)
                .saturating_sub(state.vscroll.offset),
        );

        if y as u16 >= state.inner.y {
            buf.set_style(Rect::new(state.inner.x, y as u16, 1, 1), focus_style);
        }
    }

    *widget.para.borrow_mut().deref_mut() = para;
}

impl HasFocus for ParagraphState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for ParagraphState {
    fn relocate(&mut self, offset: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, offset, clip);
        self.inner = relocate_area(self.inner, offset, clip);
        self.hscroll.relocate(offset, clip);
        self.vscroll.relocate(offset, clip);
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
            lines: 0,
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
                ct_event!(keycode press PageUp) => {
                    self.scroll_up(self.vscroll.page_len() / 2).into()
                }
                ct_event!(keycode press PageDown) => {
                    self.scroll_down(self.vscroll.page_len() / 2).into()
                }
                ct_event!(keycode press Home) => self.set_line_offset(0).into(),
                ct_event!(keycode press End) => {
                    self.set_line_offset(self.vscroll.max_offset()).into()
                }

                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press ALT-PageUp) => {
                    self.scroll_left(self.hscroll.page_len() / 2).into()
                }
                ct_event!(keycode press ALT-PageDown) => {
                    self.scroll_right(self.hscroll.page_len() / 2).into()
                }
                ct_event!(keycode press ALT-Home) => self.set_col_offset(0).into(),
                ct_event!(keycode press ALT-End) => {
                    self.set_col_offset(self.hscroll.max_offset()).into()
                }

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
