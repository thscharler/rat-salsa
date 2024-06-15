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
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};
use std::fmt;
use std::ops::Range;

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

impl<'a> StatefulWidgetRef for RDateInput<'a> {
    type State = RDateInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget)
    }
}

impl<'a> StatefulWidget for RDateInput<'a> {
    type State = RDateInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget)
    }
}

impl Default for RDateInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
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

    /// Renders the widget in invalid style.
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.widget.set_invalid(invalid);
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn get_invalid(&self) -> bool {
        self.widget.get_invalid()
    }

    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: usize) {
        self.widget.set_offset(offset)
    }

    /// Cursor position
    #[inline]
    pub fn cursor(&self) -> usize {
        self.widget.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) -> bool {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at decimal separator, if any. 0 otherwise.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.widget.set_default_cursor()
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> usize {
        self.widget.anchor()
    }

    /// Create a default value according to the mask.
    #[inline]
    pub fn default_value(&self) -> String {
        self.widget.default_value()
    }

    #[inline]
    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        self.widget.value()
    }

    #[inline]
    pub fn set_value(&mut self, date: NaiveDate) {
        self.widget.set_value(date)
    }

    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Length in grapheme count.
    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Selection
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) -> bool {
        self.widget.set_selection(anchor, cursor)
    }

    /// Select all text.
    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    /// Selection
    #[inline]
    pub fn selected_value(&self) -> &str {
        self.widget.selected_value()
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.widget.insert_char(c)
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn delete_range(&mut self, range: Range<usize>) -> bool {
        self.widget.delete_range(range)
    }

    /// Deletes the next word.
    #[inline]
    pub fn delete_next_word(&mut self) -> bool {
        self.widget.delete_next_word()
    }

    /// Deletes the given range.
    #[inline]
    pub fn delete_prev_word(&mut self) -> bool {
        self.widget.delete_prev_word()
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        self.widget.delete_prev_char()
    }

    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        self.widget.delete_next_char()
    }

    #[inline]
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next_word(extend_selection)
    }

    #[inline]
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next_word(extend_selection)
    }
    /// Move to the next char.
    #[inline]
    pub fn move_to_next(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next(extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_to_prev(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_prev(extend_selection)
    }

    /// Start of line
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_start(extend_selection)
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_end(extend_selection)
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn to_screen_col(&self, pos: usize) -> Option<u16> {
        self.widget.to_screen_col(pos)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    pub fn from_screen_col(&self, x: usize) -> Option<usize> {
        self.widget.from_screen_col(x)
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: isize, extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(cursor, extend_selection)
    }

    /// Screen position of the cursor for rendering.
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
        &self.widget.widget.focus
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
