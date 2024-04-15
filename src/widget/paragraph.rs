use crate::widget::{HasVerticalScroll, Scroll};
use crate::{ControlUI, DefaultKeys, HandleCrossterm, Input, MouseOnly};
use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Position, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{
    Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, Wrap,
};

#[derive(Debug)]
pub struct ParagraphExt<'a> {
    pub para: Paragraph<'a>,
    pub len: usize,
}

#[derive(Debug, Default)]
pub struct ParagraphExtState {
    pub area: Rect,
    pub scroll: Scroll,
}

impl HasVerticalScroll for ParagraphExtState {
    fn vscroll(&self) -> &Scroll {
        &self.scroll
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
            len,
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
        state.scroll.set_len(self.len);
        state.scroll.set_page(area.height as usize);

        let para = self.para.scroll((state.voffset() as u16, 0));
        para.render(area, buf);

        let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scroll_state = ScrollbarState::new(state.vlen()).position(state.voffset());
        scroll.render(area, buf, &mut scroll_state);
    }
}

#[derive(Debug)]
pub enum InputRequest {
    /// Select first row
    First,
    /// Select last row
    Last,
    /// Select new row
    Down(usize),
    /// Select prev row
    Up(usize),
}

impl<A, E> Input<ControlUI<A, E>> for ParagraphExtState {
    type Request = InputRequest;

    fn perform(&mut self, req: Self::Request) -> ControlUI<A, E> {
        match req {
            InputRequest::First => {
                self.vscroll().offset.set(0);
                ControlUI::Change
            }
            InputRequest::Last => {
                self.vscroll().offset.set(self.vscroll().len.get());
                ControlUI::Change
            }
            InputRequest::Down(n) => {
                self.vscroll_down(n);
                ControlUI::Change
            }
            InputRequest::Up(n) => {
                self.vscroll_up(n);
                ControlUI::Change
            }
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for ParagraphExtState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        self.handle(event, MouseOnly)
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ParagraphExtState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let req = match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(scr_area) = self.area.columns().last() {
                    if scr_area.contains(Position::new(*column, *row)) {
                        let row = row.saturating_sub(self.area.y) as usize;
                        let height = scr_area.height.saturating_sub(1) as usize;

                        let pos = (row * self.vlen()) / height;

                        self.vscroll().mouse.set(true);
                        self.vscroll().set_offset(pos);

                        Err(ControlUI::Change)
                    } else {
                        Err(ControlUI::Continue)
                    }
                } else {
                    Err(ControlUI::Continue)
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if self.vscroll().mouse.get() {
                    self.vscroll().mouse.set(false);
                }
                Err(ControlUI::Continue)
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                row,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if let Some(scr_area) = self.area.columns().last() {
                    if self.scroll.mouse.get() {
                        let row = row.saturating_sub(self.area.y) as usize;
                        let height = scr_area.height.saturating_sub(1) as usize;

                        let pos = (row * self.vlen()) / height;

                        self.vscroll().mouse.set(true);
                        self.vscroll().set_offset(pos);
                        Err(ControlUI::Change)
                    } else {
                        Err(ControlUI::Continue)
                    }
                } else {
                    Err(ControlUI::Continue)
                }
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Ok(InputRequest::Down((self.area.height / 5) as usize))
                } else {
                    Err(ControlUI::Continue)
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Ok(InputRequest::Up((self.area.height / 5) as usize))
                } else {
                    Err(ControlUI::Continue)
                }
            }
            _ => Err(ControlUI::Continue),
        };

        match req {
            Ok(req) => self.perform(req),
            Err(res) => res,
        }
    }
}
