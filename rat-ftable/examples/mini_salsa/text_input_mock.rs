#![allow(dead_code)]

use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct TextInputMock<'a> {
    style: Style,
    focus_style: Style,
    phantom_data: PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
pub struct TextInputMockState {
    pub focus: FocusFlag,
    pub area: Rect,
}

impl<'a> TextInputMock<'a> {
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
        for y in area.top()..area.bottom() {
            for x in area.top()..area.bottom() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                }
            }
        }
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

impl HasFocus for TextInputMockState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
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
