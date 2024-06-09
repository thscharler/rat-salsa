//!
//! A widget for number-input using [format_num_pattern](https://docs.rs/format_num_pattern/latest)
//!

use crate::_private::NonExhaustive;
use format_num_pattern::NumberFmtError;
use pure_rust_locales::Locale;
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_input::masked_input::MaskedInputStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::{Debug, Display, LowerExp};
use std::str::FromStr;

use crate::event::{FocusKeys, HandleEvent, MouseOnly};
pub use rat_input::date_input::ConvenientKeys;
use rat_input::event::{ReadOnly, TextOutcome};

/// Widget for numbers.
#[derive(Debug, Default, Clone)]
pub struct RNumberInput<'a> {
    widget: rat_input::number_input::NumberInput<'a>,
}

/// State.
///
/// Use `NumberInputState::new(_pattern_)` to set the date pattern.
///
#[derive(Debug, Clone)]
pub struct RNumberInputState {
    /// Base line widget.
    pub widget: rat_input::number_input::NumberInputState,
    /// Focus
    pub focus: FocusFlag,
    /// Valid flag
    pub invalid: bool,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> RNumberInput<'a> {
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

impl<'a> StatefulWidget for RNumberInput<'a> {
    type State = RNumberInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .invalid(state.invalid)
            .render(area, buf, &mut state.widget)
    }
}

impl Default for RNumberInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            invalid: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl RNumberInputState {
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, NumberFmtError> {
        Ok(Self {
            widget: rat_input::number_input::NumberInputState::new(pattern)?,
            ..Default::default()
        })
    }

    #[inline]
    pub fn new_loc<S: AsRef<str>>(pattern: S, locale: Locale) -> Result<Self, NumberFmtError> {
        Ok(Self {
            widget: rat_input::number_input::NumberInputState::new_loc(pattern, locale)?,
            ..Default::default()
        })
    }

    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// format_num_pattern format string.
    #[inline]
    pub fn format(&self) -> &str {
        self.widget.format()
    }

    /// format_num_pattern locale.
    #[inline]
    pub fn locale(&self) -> Locale {
        self.widget.locale()
    }

    /// format_num_pattern format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    #[inline]
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), NumberFmtError> {
        self.widget.set_format(pattern)
    }

    /// format_num_pattern format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    #[inline]
    pub fn set_format_loc<S: AsRef<str>>(
        &mut self,
        pattern: S,
        locale: Locale,
    ) -> Result<(), NumberFmtError> {
        self.widget.set_format_loc(pattern, locale)
    }

    #[inline]
    pub fn value<T: FromStr>(&self) -> Result<T, NumberFmtError> {
        self.widget.value()
    }

    #[inline]
    pub fn set_value<T: LowerExp + Display + Debug>(
        &mut self,
        number: T,
    ) -> Result<(), NumberFmtError> {
        self.widget.set_value(number)
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

impl HasFocusFlag for RNumberInputState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, TextOutcome> for RNumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> TextOutcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for RNumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        if self.is_focused() {
            self.widget.handle(event, ReadOnly)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for RNumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
