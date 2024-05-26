use crate::_private::NonExhaustive;
use chrono::NaiveDate;
use rat_event::util::Outcome;
use rat_event::{FocusKeys, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_input::masked_input::MaskedInputStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};
use std::fmt;

pub use rat_input::date_input::ConvenientKeys;

#[derive(Debug, Default, Clone)]
pub struct DateInput<'a> {
    widget: rat_input::date_input::DateInput<'a>,
}

#[derive(Debug, Clone)]
pub struct DateInputState {
    pub widget: rat_input::date_input::DateInputState,
    pub focus: FocusFlag,
    pub valid: bool,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> DateInput<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the compact form, if the focus is not with this widget.
    #[inline]
    pub fn show_compact(mut self, show_compact: bool) -> Self {
        self.widget = self.widget.show_compact(show_compact);
        self
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: MaskedInputStyle) -> Self {
        self.widget = self.widget.styles(style);
        self
    }

    /// Base text style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style for selection
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Style for the invalid indicator.
    #[inline]
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.invalid_style(style);
        self
    }

    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> StatefulWidget for DateInput<'a> {
    type State = DateInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .valid(state.valid)
            .render(area, buf, &mut state.widget)
    }
}

impl<'a> StatefulWidgetRef for DateInput<'a> {
    type State = DateInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .clone()
            .focused(state.is_focused())
            .valid(state.valid)
            .render(area, buf, &mut state.widget)
    }
}

impl Default for DateInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            valid: true,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl DateInputState {
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, fmt::Error> {
        Ok(Self {
            widget: rat_input::date_input::DateInputState::new(pattern)?,
            ..Default::default()
        })
    }

    #[inline]
    pub fn new_localized<S: AsRef<str>>(
        pattern: S,
        locale: chrono::Locale,
    ) -> Result<Self, fmt::Error> {
        Ok(Self {
            widget: rat_input::date_input::DateInputState::new_localized(pattern, locale)?,
            ..Default::default()
        })
    }

    /// Reset to empty.
    #[inline]
    pub fn reset(&mut self) {
        self.widget.reset();
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    #[inline]
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), fmt::Error> {
        self.widget.set_format(pattern)
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    #[inline]
    pub fn set_formats<S: AsRef<str>>(
        &mut self,
        pattern: S,
        locale: chrono::Locale,
    ) -> Result<(), fmt::Error> {
        self.widget.set_format_loc(pattern, locale)
    }

    #[inline]
    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        self.widget.value()
    }

    #[inline]
    pub fn set_value(&mut self, date: NaiveDate) {
        self.widget.set_value(date)
    }

    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all()
    }

    #[inline]
    pub fn screen_cursor(&self) -> Option<Position> {
        self.widget.screen_cursor()
    }
}

impl HasFocusFlag for DateInputState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.widget.inner
    }
}

impl HandleEvent<crossterm::event::Event, ConvenientKeys, Outcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ConvenientKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, ConvenientKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for DateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}
