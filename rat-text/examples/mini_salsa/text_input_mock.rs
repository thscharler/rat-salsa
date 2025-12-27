#![allow(dead_code)]

use crate::mini_salsa::_private::NonExhaustive;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{RelocatableState, relocate_area};
use rat_text::HasScreenCursor;
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
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

pub struct TextInputMockStyle {
    pub style: Option<Style>,
    pub focus: Option<Style>,
    pub non_exhaustive: NonExhaustive,
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

    pub fn styles(mut self, styles: TextInputMockStyle) -> Self {
        if let Some(style) = styles.style {
            self.style = style;
        }
        if let Some(focus) = styles.focus {
            self.focus_style = focus;
        }
        self
    }
}

impl Default for TextInputMockStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

pub fn textinput_mock_style(theme: &SalsaTheme) -> TextInputMockStyle {
    TextInputMockStyle {
        style: Some(theme.style(Style::INPUT)),
        focus: Some(theme.style(Style::INPUT_FOCUS)),
        non_exhaustive: NonExhaustive,
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

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.focus = z.focus.with_name(name);
        z
    }

    pub fn clear_areas(&mut self) {
        self.area = Default::default()
    }
}

impl HandleEvent<Event, Regular, Outcome> for TextInputMockState {
    fn handle(&mut self, _event: &Event, _keymap: Regular) -> Outcome {
        Outcome::Continue
    }
}

impl HandleEvent<Event, MouseOnly, Outcome> for TextInputMockState {
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> Outcome {
        Outcome::Continue
    }
}
