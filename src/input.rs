//!
//! Text input widget.
//!
//! * Can do the usual insert/delete/movement operations.
//! * Text selection via keyboard and mouse.
//! * Scrolls with the cursor.
//! * Modes for focused and valid.
//!
//!
//! The visual cursor must be set separately after rendering.
//! It is accessible as [RTextInputState::screen_cursor()] after rendering.
//!
//! For event-handling call one of the HandleEvent implementations.
//!

use crate::_private::NonExhaustive;
use crate::event::{FocusKeys, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_input::event::{ReadOnly, TextOutcome};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::{Block, StatefulWidgetRef};
use std::ops::Range;

pub use rat_input::input::core;
pub use rat_input::input::TextInputStyle;

/// Text input widget.
#[derive(Debug, Default, Clone)]
pub struct RTextInput<'a> {
    widget: rat_input::input::TextInput<'a>,
}

#[derive(Debug, Clone)]
pub struct RTextInputState {
    pub widget: rat_input::input::TextInputState,
    pub non_exhaustive: NonExhaustive,
}

impl<'a> RTextInput<'a> {
    pub fn new() -> Self {
        Self::default()
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
    /// This is patched onto either base_style or focus_style
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

impl<'a> StatefulWidgetRef for RTextInput<'a> {
    type State = RTextInputState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget)
    }
}

impl<'a> StatefulWidget for RTextInput<'a> {
    type State = RTextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget)
    }
}

impl Default for RTextInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for RTextInputState {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.widget.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl RTextInputState {
    pub fn new() -> Self {
        Self::default()
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
    pub fn clear(&mut self) -> bool {
        self.widget.clear()
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

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> usize {
        self.widget.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) -> bool {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> usize {
        self.widget.anchor()
    }

    /// Text.
    #[inline]
    pub fn value(&self) -> &str {
        self.widget.value()
    }

    /// Set text.
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.widget.set_value(s)
    }

    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Text length as grapheme count.
    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection.
    #[inline]
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) -> bool {
        self.widget.set_selection(anchor, cursor)
    }

    /// Selection.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.widget.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_value(&self) -> &str {
        self.widget.selected_value()
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.widget.insert_char(c)
    }

    /// Deletes the given range.
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
        self.widget.move_to_prev_word(extend_selection)
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

    // End of line
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

    /// Set the cursor position from a visual position relative to the origin.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: isize, extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(cursor, extend_selection)
    }

    /// The current text cursor as an absolute screen position.
    #[inline]
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            self.widget.screen_cursor()
        } else {
            None
        }
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, TextOutcome> for RTextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> TextOutcome {
        if self.gained_focus() {
            TextOutcome::NotUsed
        } else if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for RTextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        if self.gained_focus() {
            TextOutcome::NotUsed
        } else if self.is_focused() {
            self.widget.handle(event, ReadOnly)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for RTextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        if self.gained_focus() {
            TextOutcome::NotUsed
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}
