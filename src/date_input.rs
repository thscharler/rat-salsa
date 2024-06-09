//!
//! A widget for date-input using [chrono](https://docs.rs/chrono/latest/chrono/)
//!

use crate::_private::NonExhaustive;
use chrono::NaiveDate;
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_input::masked_input::MaskedInputStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt;

use crate::event::{FocusKeys, HandleEvent, MouseOnly};
pub use rat_input::date_input::ConvenientKeys;
use rat_input::event::{ReadOnly, TextOutcome};

/// Widget for dates.
#[derive(Debug, Default, Clone)]
pub struct RDateInput<'a> {
    widget: rat_input::date_input::DateInput<'a>,
}

/// State.
///
/// Use `DateInputState::new(_pattern_)` to set the date pattern.
///
#[derive(Debug, Clone)]
pub struct RDateInputState {
    /// Base line widget.
    pub widget: rat_input::date_input::DateInputState,
    /// Focus
    pub focus: FocusFlag,
    /// Valid flag
    pub invalid: bool,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> RDateInput<'a> {
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

impl<'a> StatefulWidget for RDateInput<'a> {
    type State = RDateInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .invalid(state.invalid)
            .render(area, buf, &mut state.widget)
    }
}

impl Default for RDateInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            invalid: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl RDateInputState {
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, fmt::Error> {
        Ok(Self {
            widget: rat_input::date_input::DateInputState::new(pattern)?,
            ..Default::default()
        })
    }

    #[inline]
    pub fn new_loc<S: AsRef<str>>(pattern: S, locale: chrono::Locale) -> Result<Self, fmt::Error> {
        Ok(Self {
            widget: rat_input::date_input::DateInputState::new_loc(pattern, locale)?,
            ..Default::default()
        })
    }

    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// chrono format string.
    #[inline]
    pub fn format(&self) -> &str {
        self.widget.format()
    }

    /// chrono locale.
    #[inline]
    pub fn locale(&self) -> chrono::Locale {
        self.widget.locale()
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
    pub fn set_format_loc<S: AsRef<str>>(
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
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            self.widget.screen_cursor()
        } else {
            None
        }
    }
}

impl HasFocusFlag for RDateInputState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, ConvenientKeys, TextOutcome> for RDateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ConvenientKeys) -> TextOutcome {
        if self.is_focused() {
            self.widget.handle(event, ConvenientKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, TextOutcome> for RDateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> TextOutcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for RDateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        if self.is_focused() {
            self.widget.handle(event, ReadOnly)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for RDateInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
