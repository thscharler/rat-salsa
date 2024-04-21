use crate::HasVerticalScroll;
use crate::{ControlUI, DefaultKeys, HandleCrossterm, MouseOnly};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};

#[derive(Debug)]
pub struct ParagraphExt<'a> {
    pub para: Paragraph<'a>,
    pub vlen: usize,
}

#[derive(Debug, Default)]
pub struct ParagraphExtState {
    pub area: Rect,
    pub vlen: usize,
    pub voffset: usize,
}

impl HasVerticalScroll for ParagraphExtState {
    fn vlen(&self) -> usize {
        self.vlen
    }

    fn voffset(&self) -> usize {
        self.voffset
    }

    fn set_voffset(&mut self, offset: usize) {
        self.voffset = offset;
    }

    fn vpage(&self) -> usize {
        self.area.height as usize
    }
}

impl<'a> ParagraphExt<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        let t = text.into();
        let len = t.height();
        Self {
            para: Paragraph::new(t),
            vlen: len,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.para = self.para.block(block);
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.para = self.para.style(style);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.para = self.para.wrap(wrap);
        self
    }

    pub fn scroll(mut self, offset: (u16, u16)) -> Self {
        self.para = self.para.scroll(offset);
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
        state.vlen = self.vlen;

        let para = self.para.scroll((state.voffset() as u16, 0));
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
