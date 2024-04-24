use crate::lib_scroll::{HasScrolling, ScrollParam, ScrolledWidget};
///
/// Adapter for ratatui::widget::Paragraph
///
use crate::{ControlUI, DefaultKeys, HandleCrossterm, MouseOnly};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use std::cell::Cell;
use std::cmp::min;

#[derive(Debug)]
pub struct ParagraphExt<'a> {
    pub para: Paragraph<'a>,
    pub block: Option<Block<'a>>,
    pub wrap: Option<Wrap>,

    // maybe??
    pub cached_area_width: Cell<u16>,
    pub cached_line_width: Cell<usize>,
    pub cached_line_count: Cell<usize>,

    pub non_exhaustive: (),
}

#[derive(Debug, Default)]
pub struct ParagraphExtState {
    pub area: Rect,
    pub para_area: Rect,

    pub vlen: usize,
    pub hlen: usize,
    pub voffset: usize,
    pub hoffset: usize,

    pub non_exhaustive: (),
}

impl<'a> ScrolledWidget for ParagraphExt<'a> {
    fn need_scroll(&self, mut area: Rect, _state: &mut Self::State) -> ScrollParam {
        area = self.block.inner_if_some(area);

        self.cached_area_width.set(area.width);

        let show_horizontal = if self.wrap.is_some() {
            false
        } else {
            let width = self.para.line_width();
            self.cached_line_width.set(width);

            width >= area.width as usize
        };

        let lines = self.para.line_count(area.width);
        self.cached_line_count.set(lines);
        let show_vertical = lines > area.height as usize;

        ScrollParam {
            has_hscroll: show_horizontal,
            has_vscroll: show_vertical,
        }
    }
}

impl HasScrolling for ParagraphExtState {
    fn max_v_offset(&self) -> usize {
        self.vlen.saturating_sub(self.para_area.height as usize)
    }

    fn max_h_offset(&self) -> usize {
        self.hlen.saturating_sub(self.para_area.width as usize)
    }

    fn v_page_len(&self) -> usize {
        self.para_area.height as usize
    }

    fn h_page_len(&self) -> usize {
        self.para_area.width as usize
    }

    fn v_offset(&self) -> usize {
        self.voffset
    }

    fn h_offset(&self) -> usize {
        self.hoffset
    }

    fn set_v_offset(&mut self, offset: usize) {
        self.voffset = min(offset, self.vlen);
    }

    fn set_h_offset(&mut self, offset: usize) {
        self.hoffset = min(offset, self.hlen);
    }
}

impl<'a> ParagraphExt<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        let t = text.into();
        Self {
            para: Paragraph::new(t),
            block: None,
            wrap: None,
            cached_area_width: Cell::new(0),
            cached_line_width: Cell::new(0),
            cached_line_count: Cell::new(0),
            non_exhaustive: (),
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

impl<'a> StatefulWidget for ParagraphExt<'a> {
    type State = ParagraphExtState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.para_area = self.block.inner_if_some(area);

        if self.cached_area_width.get() == state.para_area.width {
            state.hlen = self.cached_line_width.get();
            state.vlen = self.cached_line_count.get();
        } else {
            state.hlen = self.para.line_width();
            state.vlen = self.para.line_count(state.para_area.width);
        }

        let para = self
            .para
            .scroll((state.v_offset() as u16, state.h_offset() as u16));
        para.render(area, buf);
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ParagraphExtState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        self.handle(event, MouseOnly)
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ParagraphExtState {
    fn handle(&mut self, _event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}
