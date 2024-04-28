//!
//! Text input widget.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection.
//! * Scrolls with the cursor.
//! * Can set the cursor or use its own block cursor.
//! * Can show an indicator for invalid input.

use crate::_private::NonExhaustive;
use crate::{ControlUI, ValidFlag};
use crate::{DefaultKeys, FrameWidget, HandleCrossterm, MouseOnly};
use crate::{FocusFlag, HasFocusFlag, HasValidFlag};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_input::input::{TextInput, TextInputState, TextInputStyle};
use rat_input::{input, Outcome};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;
use std::ops::Range;

/// Text input widget.
#[derive(Debug)]
pub struct TextInputExt<'a> {
    widget: TextInput<'a>,
}

impl<'a> Default for TextInputExt<'a> {
    fn default() -> Self {
        Self {
            widget: Default::default(),
        }
    }
}

impl<'a> TextInputExt<'a> {
    /// Set the combined style.
    pub fn style(mut self, style: TextInputStyle) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Base text style.
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.base_style(style);
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.invalid_style(style);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> FrameWidget for TextInputExt<'a> {
    type State = TextInputExtState;

    fn render(mut self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        state.area = area;
        state.widget.value.set_width(state.area.width as usize);

        self.widget = self
            .widget
            .focused(state.is_focused())
            .valid(state.is_valid());

        frame.render_stateful_widget(self.widget, area, &mut state.widget);

        if state.is_focused() {
            frame.set_cursor(state.widget.cursor.x, state.widget.cursor.y);
        }
    }
}

/// Input state data.
#[derive(Debug, Clone)]
pub struct TextInputExtState {
    /// Widget state
    pub widget: TextInputState,
    /// Focus
    pub focus: FocusFlag,
    /// Valid.
    pub valid: ValidFlag,
    /// Area
    pub area: Rect,
    ///
    pub non_exhaustive: NonExhaustive,
}

impl Default for TextInputExtState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            valid: Default::default(),
            area: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for TextInputExtState {
    #[allow(non_snake_case)]
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let focused = self.is_focused();
        match input::handle_events(&mut self.widget, focused, event) {
            Outcome::Unused => ControlUI::Continue,
            Outcome::Unchanged => ControlUI::NoChange,
            Outcome::Changed => ControlUI::Change,
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TextInputExtState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        match input::handle_mouse_events(&mut self.widget, event) {
            Outcome::Unused => ControlUI::Continue,
            Outcome::Unchanged => ControlUI::NoChange,
            Outcome::Changed => ControlUI::Change,
        }
    }
}

impl TextInputExtState {
    /// Reset to empty.
    pub fn reset(&mut self) {
        self.widget.reset();
    }

    /// Offset shown.
    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    pub fn set_offset(&mut self, offset: usize) {
        self.widget.set_offset(offset);
    }

    /// Set the cursor position, reset selection.
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) {
        self.widget.set_cursor(cursor, extend_selection);
    }

    /// Cursor position.
    pub fn cursor(&self) -> usize {
        self.widget.cursor()
    }

    /// Set text.
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.widget.set_value(s);
    }

    /// Text.
    pub fn value(&self) -> &str {
        self.widget.value()
    }

    /// Text
    pub fn as_str(&self) -> &str {
        self.widget.as_str()
    }

    /// Empty.
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Text length as grapheme count.
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Selection.
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection.
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.widget.set_selection(anchor, cursor);
    }

    /// Selection.
    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    /// Selection.
    pub fn selection(&self) -> Range<usize> {
        self.widget.selection()
    }

    /// Selection.
    pub fn selection_str(&self) -> &str {
        self.widget.selection_str()
    }

    /// Previous word boundary
    pub fn prev_word_boundary(&self) -> usize {
        self.widget.prev_word_boundary()
    }

    /// Next word boundary
    pub fn next_word_boundary(&self) -> usize {
        self.widget.next_word_boundary()
    }

    /// Set the cursor position from a visual position relative to the origin.
    pub fn set_offset_relative_cursor(&mut self, rpos: isize, extend_selection: bool) {
        self.widget
            .set_offset_relative_cursor(rpos, extend_selection);
    }

    /// Move to the next char.
    pub fn move_to_next(&mut self, extend_selection: bool) {
        self.widget.move_to_next(extend_selection);
    }

    /// Move to the previous char.
    pub fn move_to_prev(&mut self, extend_selection: bool) {
        self.widget.move_to_prev(extend_selection);
    }

    /// Insert a char a the current position.
    pub fn insert_char(&mut self, c: char) {
        self.widget.insert_char(c);
    }

    /// Replace the given range with a new string.
    pub fn replace(&mut self, range: Range<usize>, new: &str) {
        self.widget.replace(range, new);
    }

    /// Delete the char before the cursor.
    pub fn delete_prev_char(&mut self) {
        self.widget.delete_prev_char();
    }

    /// Delete the char after the cursor.
    pub fn delete_next_char(&mut self) {
        self.widget.delete_next_char();
    }
}

impl HasFocusFlag for TextInputExtState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasValidFlag for TextInputExtState {
    fn valid(&self) -> &ValidFlag {
        &self.valid
    }
}
