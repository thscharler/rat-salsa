use crate::_private::NonExhaustive;
use crate::events::Outcome;
use crate::{ScrollingOutcome, ScrollingState, ScrollingWidget};
use rat_event::{FocusKeys, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use std::cmp::min;

///
/// Adapter for ratatui::widget::Paragraph
///

#[derive(Debug)]
pub struct ParagraphS<'a> {
    para: Paragraph<'a>,
    block: Option<Block<'a>>,
    wrap: Option<Wrap>,
}

#[derive(Debug)]
pub struct ParagraphSState {
    pub area: Rect,
    pub para_area: Rect,

    pub v_len: usize,
    pub h_len: usize,
    pub v_offset: usize,
    pub h_offset: usize,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for ParagraphS<'a> {
    fn default() -> Self {
        Self {
            para: Default::default(),
            block: None,
            wrap: None,
        }
    }
}

impl<'a, State> ScrollingWidget<State> for ParagraphS<'a> {
    fn need_scroll(&self, mut area: Rect, _: &mut State) -> (bool, bool) {
        area = self.block.inner_if_some(area);

        let show_horizontal = if self.wrap.is_some() {
            false
        } else {
            let width = self.para.line_width();
            width >= area.width as usize
        };

        let lines = self.para.line_count(area.width);
        let show_vertical = lines > area.height as usize;

        (show_horizontal, show_vertical)
    }
}

impl Default for ParagraphSState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            para_area: Default::default(),
            v_len: 0,
            h_len: 0,
            v_offset: 0,
            h_offset: 0,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollingState for ParagraphSState {
    fn vertical_max_offset(&self) -> usize {
        self.v_len.saturating_sub(self.para_area.height as usize)
    }

    fn vertical_offset(&self) -> usize {
        self.v_offset
    }

    fn vertical_page(&self) -> usize {
        self.para_area.height as usize
    }

    fn vertical_scroll(&self) -> usize {
        self.para_area.height as usize / 10
    }

    fn horizontal_max_offset(&self) -> usize {
        self.h_len.saturating_sub(self.para_area.width as usize)
    }

    fn horizontal_offset(&self) -> usize {
        self.h_offset
    }

    fn horizontal_page(&self) -> usize {
        self.para_area.width as usize
    }

    fn horizontal_scroll(&self) -> usize {
        self.para_area.width as usize / 10
    }

    fn set_vertical_offset(&mut self, offset: usize) -> ScrollingOutcome {
        let old_offset = min(self.v_offset, self.v_len.saturating_sub(1));
        let new_offset = min(offset, self.v_len.saturating_sub(1));

        self.v_offset = new_offset;

        if new_offset > old_offset {
            ScrollingOutcome::Scrolled
        } else {
            ScrollingOutcome::Denied
        }
    }

    fn set_horizontal_offset(&mut self, offset: usize) -> ScrollingOutcome {
        let old_offset = min(self.h_offset, self.h_len.saturating_sub(1));
        let new_offset = min(offset, self.h_len.saturating_sub(1));

        self.h_offset = new_offset;

        if new_offset > old_offset {
            ScrollingOutcome::Scrolled
        } else {
            ScrollingOutcome::Denied
        }
    }
}

impl<'a> ParagraphS<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        let t = text.into();
        Self {
            para: Paragraph::new(t),
            ..Self::default()
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block.clone());
        self.para = self.para.block(block);
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.para = self.para.style(style);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);
        self.para = self.para.wrap(wrap);
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.para = self.para.alignment(alignment);
        self
    }

    pub fn left_aligned(mut self) -> Self {
        self.para = self.para.left_aligned();
        self
    }

    pub fn centered(mut self) -> Self {
        self.para = self.para.centered();
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.para = self.para.right_aligned();
        self
    }
}

impl<'a> StatefulWidget for ParagraphS<'a> {
    type State = ParagraphSState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.para_area = self.block.inner_if_some(area);

        state.h_len = if self.wrap.is_some() {
            state.para_area.width as usize
        } else {
            self.para.line_width()
        };
        state.v_len = self.para.line_count(state.para_area.width);

        let para = self.para.scroll((
            state.vertical_offset() as u16,
            state.horizontal_offset() as u16,
        ));
        para.render(area, buf);
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for ParagraphSState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        Outcome::NotUsed
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ParagraphSState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        Outcome::NotUsed
    }
}
