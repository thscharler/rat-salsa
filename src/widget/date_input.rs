use crate::lib_focus::Validate;
use crate::widget::mask_input::{MaskedInput, MaskedInputState, MaskedInputStyle};
use crate::{ControlUI, DefaultKeys, FocusFlag, FrameWidget, HandleCrossterm};
use chrono::NaiveDate;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::Style;
use ratatui::Frame;
use std::fmt::Debug;

#[derive(Debug)]
pub struct DateInput {
    pub input: MaskedInput,
}

impl Default for DateInput {
    fn default() -> Self {
        Self {
            input: Default::default(),
        }
    }
}

impl DateInput {
    /// Use extra insets for the text input.
    pub fn insets(mut self, insets: Margin) -> Self {
        self.input = self.input.insets(insets);
        self
    }

    /// Use our own cursor indicator or the terminal cursor.
    pub fn terminal_cursor(mut self, terminal: bool) -> Self {
        self.input = self.input.terminal_cursor(terminal);
        self
    }

    /// Do accept keyboard events event without being focused.
    /// Useful for a catch field, eg "find stuff"
    pub fn without_focus(mut self, without_focus: bool) -> Self {
        self.input = self.input.without_focus(without_focus);
        self
    }

    /// Set the combined style.
    pub fn style(mut self, style: MaskedInputStyle) -> Self {
        self.input = self.input.style(style);
        self
    }

    /// Base text style.
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.base_style(style);
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.focus_style(style);
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.select_style(style);
        self
    }

    /// Style for our own cursor.
    pub fn cursor_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.cursor_style(style);
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.invalid_style(style);
        self
    }

    /// Marker character for invalid field.
    pub fn invalid_char(mut self, invalid: char) -> Self {
        self.input = self.input.invalid_char(invalid);
        self
    }
}

impl FrameWidget for DateInput {
    type State = DateInputState;

    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        self.input.render(frame, area, &mut state.input);
    }
}

#[derive(Debug)]
pub struct DateInputState {
    pub input: MaskedInputState,
    pub format: String,
}

impl Default for DateInputState {
    fn default() -> Self {
        Self {
            input: Default::default(),
            format: Default::default(),
        }
    }
}

impl DateInputState {
    /// Reset to empty.
    pub fn reset(&mut self) {
        self.input.reset();
    }

    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.input.set_display_mask(s);
    }

    pub fn set_mask<S: Into<String>>(&mut self, s: S) {
        self.input.set_mask(s);
    }

    pub fn set_format<S: Into<String>>(&mut self, s: S) {
        self.format = s.into();
    }

    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(self.input.compact_value().as_str(), self.format.as_str())
    }

    pub fn set_value(&mut self, date: NaiveDate) {
        let v = date.format(self.format.as_str()).to_string();
        self.input.set_value(v);
    }

    pub fn select_all(&mut self) {
        self.input.select_all()
    }
}

impl<A: Debug, E: Debug> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for DateInputState {
    fn handle(&mut self, event: &Event, keymap: DefaultKeys) -> ControlUI<A, E> {
        let r = self.input.handle(event, keymap);
        r.on_change_do(|| {
            self.input.valid = self.value().is_ok();
        });
        r
    }
}

impl Validate for DateInputState {
    fn focus(&self) -> &FocusFlag {
        &self.input.focus
    }

    fn validate(&mut self) {
        if let Ok(d) = self.value() {
            self.set_value(d);
            self.select_all();
        }
    }
}
