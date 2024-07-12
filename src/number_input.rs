//!
//! Widget for number input
//!

use crate::_private::NonExhaustive;
use crate::event::{ReadOnly, TextOutcome};
use crate::input::TextInputStyle;
use crate::masked_input::{MaskedInput, MaskedInputState};
use format_num_pattern::{NumberFmtError, NumberFormat, NumberSymbols};
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};
use std::fmt::{Debug, Display, LowerExp};
use std::ops::Range;
use std::str::FromStr;

/// Numeric input.
///
/// Uses [format_num_pattern](https://docs.rs/format_num_pattern/latest/format_num_pattern/index.html)
/// for the actual formatting/parsing and [MaskedInput] for the rendering.
///
#[derive(Debug, Default, Clone)]
pub struct NumberInput<'a> {
    widget: MaskedInput<'a>,
}

#[derive(Debug, Clone)]
pub struct NumberInputState {
    pub widget: MaskedInputState,
    /// NumberFormat pattern.
    pattern: String,
    /// Locale
    locale: format_num_pattern::Locale,
    // MaskedInput internally always works with the POSIX locale.
    // So don't be surprised, if you see that one instead of the
    // paramter locale used here.
    format: NumberFormat,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> NumberInput<'a> {
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
    pub fn styles(mut self, style: TextInputStyle) -> Self {
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

impl<'a> StatefulWidgetRef for NumberInput<'a> {
    type State = NumberInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget);
    }
}

impl<'a> StatefulWidget for NumberInput<'a> {
    type State = NumberInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget);
    }
}

impl HasFocusFlag for NumberInputState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.widget.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl Default for NumberInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            pattern: Default::default(),
            locale: Default::default(),
            format: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl NumberInputState {
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, NumberFmtError> {
        let mut s = Self::default();
        s.set_format(pattern)?;
        Ok(s)
    }

    pub fn new_loc<S: AsRef<str>>(
        pattern: S,
        locale: format_num_pattern::Locale,
    ) -> Result<Self, NumberFmtError> {
        let mut s = Self::default();
        s.set_format_loc(pattern.as_ref(), locale)?;
        Ok(s)
    }

    /// [format_num_pattern] format string.
    #[inline]
    pub fn format(&self) -> &str {
        self.pattern.as_str()
    }

    /// chrono locale.
    #[inline]
    pub fn locale(&self) -> chrono::Locale {
        self.locale
    }

    /// Set format.
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), NumberFmtError> {
        self.set_format_loc(pattern, format_num_pattern::Locale::default())
    }

    /// Set format and locale.
    pub fn set_format_loc<S: AsRef<str>>(
        &mut self,
        pattern: S,
        locale: format_num_pattern::Locale,
    ) -> Result<(), NumberFmtError> {
        let sym = NumberSymbols::monetary(locale);

        self.format = NumberFormat::new(pattern.as_ref())?;
        self.widget.set_mask(pattern.as_ref())?;
        self.widget.set_num_symbols(sym);

        Ok(())
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn set_focused(&mut self, focus: bool) {
        self.widget.focus.set(focus);
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn is_focused(&mut self) -> bool {
        self.widget.focus.get()
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.widget.invalid = invalid;
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn get_invalid(&self) -> bool {
        self.widget.invalid
    }

    /// Reset to empty
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

    /// Parses the text as the desired value type.
    pub fn value<T: FromStr>(&self) -> Result<T, NumberFmtError> {
        let s = self.widget.value();
        self.format.parse(s)
    }

    /// Sets the numeric value.
    pub fn set_value<T: LowerExp + Display + Debug>(
        &mut self,
        number: T,
    ) -> Result<(), NumberFmtError> {
        let s = self.format.fmt(number)?;
        self.widget.set_value(s);
        Ok(())
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
        self.widget.screen_cursor()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for NumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        self.widget.handle(event, Regular)
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for NumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        self.widget.handle(event, ReadOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for NumberInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut NumberInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut NumberInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut NumberInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
