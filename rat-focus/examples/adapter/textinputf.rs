#![allow(dead_code)]

use crate::adapter::_private::NonExhaustive;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct TextInputF<'a> {
    style: Style,
    focus_style: Style,
    phantom_data: PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
pub struct TextInputFState {
    pub focus: FocusFlag,
    pub area: Rect,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> TextInputF<'a> {
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

impl<'a> StatefulWidget for TextInputF<'a> {
    type State = TextInputFState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let mut text = Line::from("                    ");
        if state.is_focused() {
            text = text.style(self.focus_style);
        } else {
            text = text.style(self.style);
        }
        text.render(area, buf);
    }
}

impl Default for TextInputFState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl TextInputFState {
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        Some((self.area.x, self.area.y))
    }
}

impl HasFocus for TextInputFState {
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

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for TextInputFState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        Outcome::Continue
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TextInputFState {
    fn handle(&mut self, _event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        Outcome::Continue
    }
}
