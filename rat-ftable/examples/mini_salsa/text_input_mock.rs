#![allow(dead_code)]

use rat_cursor::HasScreenCursor;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct TextInputMock<'a> {
    style: Style,
    text: String,
    focus_style: Style,
    phantom_data: PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
pub struct TextInputMockState {
    pub focus: FocusFlag,
    pub area: Rect,
}

impl<'a> TextInputMock<'a> {
    /// Sample text
    pub fn sample(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Base text style.
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }
}

impl<'a> StatefulWidget for TextInputMock<'a> {
    type State = TextInputMockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        if state.is_focused() {
            buf.set_style(area, self.focus_style);
        } else {
            buf.set_style(area, self.style);
        }
        buf.set_stringn(
            area.x,
            area.y,
            self.text,
            area.width as usize,
            Style::default(),
        );
    }
}

impl Default for TextInputMockState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
        }
    }
}

impl HasScreenCursor for TextInputMockState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() && !self.area.is_empty() {
            Some((self.area.x, self.area.y))
        } else {
            None
        }
    }
}

impl HasFocus for TextInputMockState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for TextInputMockState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
    }
}

impl TextInputMockState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_areas(&mut self) {
        self.area = Default::default()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for TextInputMockState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        Outcome::Continue
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TextInputMockState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        Outcome::Continue
    }
}
