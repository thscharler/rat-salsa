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
//! It is accessible as [TextInputState::screen_cursor()] or [TextInputState::cursor] after rendering.
//!
//! Event handling by calling the freestanding fn [crate::masked_input::handle_events].
//! There's [handle_mouse_events] if you want to override the default key bindings but keep
//! the mouse behaviour.
//!

use crate::_private::NonExhaustive;
use rat_event::util::Outcome;
use rat_event::{FocusKeys, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;
use std::ops::Range;

pub use rat_input::input::TextInputStyle;

#[derive(Debug, Default, Clone)]
pub struct TextInput<'a> {
    widget: rat_input::input::TextInput<'a>,
}

#[derive(Debug, Clone)]
pub struct TextInputState {
    pub widget: rat_input::input::TextInputState,
    pub focus: FocusFlag,
    pub valid: bool,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> TextInput<'a> {
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

impl<'a> StatefulWidget for TextInput<'a> {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focused(state.is_focused())
            .valid(state.valid)
            .render(area, buf, &mut state.widget)
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            valid: true,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for TextInputState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset to empty.
    #[inline]
    pub fn reset(&mut self) {
        self.widget.reset()
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

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> usize {
        self.widget.cursor()
    }

    /// Set text.
    #[inline]
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.widget.set_value(s)
    }

    /// Text.
    #[inline]
    pub fn value(&self) -> &str {
        self.widget.value()
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
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.widget.set_selection(anchor, cursor)
    }

    /// Selection.
    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection.
    #[inline]
    pub fn selection_str(&self) -> &str {
        self.widget.selection_str()
    }

    /// Previous word boundary
    #[inline]
    pub fn prev_word_boundary(&self) -> usize {
        self.widget.prev_word_boundary()
    }

    /// Next word boundary
    #[inline]
    pub fn next_word_boundary(&self) -> usize {
        self.widget.next_word_boundary()
    }

    /// Set the cursor position from a visual position relative to the origin.
    #[inline]
    pub fn set_screen_cursor(&mut self, rpos: isize, extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(rpos, extend_selection)
    }

    /// The current text cursor as an absolute screen position.
    #[inline]
    pub fn screen_cursor(&self) -> Option<Position> {
        self.widget.screen_cursor()
    }

    /// Move to the next char.
    #[inline]
    pub fn move_to_next(&mut self, extend_selection: bool) {
        self.widget.move_to_next(extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_to_prev(&mut self, extend_selection: bool) {
        self.widget.move_to_prev(extend_selection)
    }

    /// Insert a char a the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) {
        self.widget.insert_char(c)
    }

    /// Replace the given range with a new string.
    #[inline]
    pub fn remove(&mut self, range: Range<usize>) {
        self.widget.remove(range)
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) {
        self.widget.delete_prev_char()
    }

    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) {
        self.widget.delete_next_char()
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}
