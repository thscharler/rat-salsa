use rat_event::util::MouseFlags;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocus};
use rat_reloc::{relocate_area, RelocatableState};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use rat_text::line_number::{LineNumberState, LineNumbers};
use rat_widget::util::revert_style;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;

#[derive(Debug, Default)]
pub struct EndlessScroll<'a> {
    style: Style,
    focus_style: Option<Style>,
    cursor_style: Option<Style>,
    v_scroll: Option<Scroll<'a>>,
    block: Option<Block<'a>>,
    max: usize,
}

#[derive(Debug, Default)]
pub struct EndlessScrollState {
    pub area: Rect,
    pub widget_area: Rect,

    pub line_num: LineNumberState,

    pub vscroll: ScrollState,

    pub focus: FocusFlag,
    pub mouse: MouseFlags,
}

impl<'a> EndlessScroll<'a> {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            focus_style: None,
            cursor_style: None,
            v_scroll: None,
            block: None,
            max: 0,
        }
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = Some(style.into());
        self
    }

    /// Base style.
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = Some(style);
        self
    }

    pub fn v_scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.v_scroll = Some(scroll);
        self
    }

    /// Block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block.style(self.style));
        self
    }
}

impl<'a> StatefulWidget for EndlessScroll<'a> {
    type State = EndlessScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let sa = ScrollArea::new()
            .block(self.block.as_ref())
            .v_scroll(self.v_scroll.as_ref());
        state.widget_area = sa.inner(area, None, Some(&state.vscroll));

        state
            .vscroll
            .set_max_offset(self.max.saturating_sub(area.height as usize));
        state.vscroll.set_page_len(area.height as usize);

        if self.block.is_none() {
            buf.set_style(area, self.style);
        }
        sa.render(
            area,
            buf,
            &mut ScrollAreaState::new().v_scroll(&mut state.vscroll),
        );

        let focus_style = self.focus_style.unwrap_or(revert_style(self.style));
        let cursor_style = if state.is_focused() {
            focus_style
        } else {
            self.cursor_style.unwrap_or(self.style)
        };

        LineNumbers::new()
            .start(state.vscroll.offset as u32)
            .cursor(state.vscroll.offset as u32 + area.height as u32 / 2)
            .cursor_style(cursor_style)
            .render(state.widget_area, buf, &mut state.line_num);
    }
}

impl HasFocus for EndlessScrollState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for EndlessScrollState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.widget_area = relocate_area(self.widget_area, shift, clip);
    }
}

impl EndlessScrollState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for EndlessScrollState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.vscroll.scroll_up(1).into(),
                ct_event!(keycode press Down) => self.vscroll.scroll_down(1).into(),
                ct_event!(keycode press PageUp) => {
                    self.vscroll.scroll_up(self.vscroll.page_len / 2).into()
                }
                ct_event!(keycode press PageDown) => {
                    self.vscroll.scroll_down(self.vscroll.page_len / 2).into()
                }
                ct_event!(keycode press Home) => self.vscroll.scroll_to_pos(0).into(),
                ct_event!(keycode press End) => {
                    self.vscroll.scroll_to_pos(self.vscroll.max_offset).into()
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        if !r.is_consumed() {
            self.handle(event, MouseOnly)
        } else {
            r
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for EndlessScrollState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> Outcome {
        match ScrollAreaState::new()
            .area(self.area)
            .v_scroll(&mut self.vscroll)
            .handle(event, MouseOnly)
        {
            ScrollOutcome::Up(n) => self.vscroll.scroll_up(n).into(),
            ScrollOutcome::Down(n) => self.vscroll.scroll_down(n).into(),
            ScrollOutcome::VPos(p) => self.vscroll.set_offset(p).into(),
            r => r.into(),
        }
    }
}
