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
use std::cmp::min;

#[derive(Debug)]
pub struct ParagraphExt<'a> {
    pub para: Paragraph<'a>,
    pub block: Option<Block<'a>>,
    pub wrap: Option<Wrap>,
    pub overscroll: usize,
}

#[derive(Debug, Default)]
pub struct ParagraphExtState {
    pub area: Rect,
    pub para_area: Rect,

    pub overscroll: usize,

    pub has_vscroll: bool,
    pub vlen: usize,
    pub voffset: usize,
    pub has_hscroll: bool,
    pub hlen: usize,
    pub hoffset: usize,
}

impl<'a> ScrolledWidget for ParagraphExt<'a> {
    fn need_scroll(&self, area: Rect) -> ScrollParam {
        let (show_horizontal, hlen) = if self.wrap.is_some() {
            (false, area.width as usize)
        } else {
            let width = self.para.line_width();
            if width >= area.width as usize {
                (true, width)
            } else {
                (false, width)
            }
        };

        let vlen = self.para.line_count(hlen as u16);
        let show_vertical = true; // always show ...

        ScrollParam {
            hlen,
            vlen,
            has_hscroll: show_horizontal,
            has_vscroll: show_vertical,
        }
    }
}

impl HasScrolling for ParagraphExtState {
    fn has_vscroll(&self) -> bool {
        self.has_vscroll
    }

    fn has_hscroll(&self) -> bool {
        self.has_hscroll
    }

    fn vlen(&self) -> usize {
        self.vlen
    }

    fn hlen(&self) -> usize {
        self.hlen
    }

    fn vmax_offset(&self) -> usize {
        self.vlen.saturating_sub(self.para_area.height as usize)
    }

    fn hmax_offset(&self) -> usize {
        self.hlen.saturating_sub(self.para_area.width as usize)
    }

    fn voffset(&self) -> usize {
        self.voffset
    }

    fn hoffset(&self) -> usize {
        self.hoffset
    }

    fn set_voffset(&mut self, offset: usize) {
        self.voffset = min(
            offset,
            self.vmax_offset() + (self.para_area.height as usize * self.overscroll / 100),
        );
    }

    fn set_hoffset(&mut self, offset: usize) {
        self.hoffset = min(
            offset,
            self.hmax_offset() + (self.para_area.width as usize * self.overscroll / 100),
        );
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
            overscroll: 0,
        }
    }

    /// Overscrolling in percent.
    pub fn overscroll(mut self, percent: usize) -> Self {
        self.overscroll = percent;
        self
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

        let scroll_config = self.need_scroll(area);
        state.overscroll = self.overscroll;
        state.hlen = scroll_config.hlen;
        state.vlen = scroll_config.vlen;
        state.has_hscroll = scroll_config.has_hscroll;
        state.has_vscroll = scroll_config.has_vscroll;

        let para = self
            .para
            .scroll((state.voffset() as u16, (state.hoffset() as u16)));
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
