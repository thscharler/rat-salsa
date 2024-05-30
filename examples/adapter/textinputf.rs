use crate::adapter::_private::NonExhaustive;
use rat_event::{FocusKeys, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_input::event::TextOutcome;
use rat_input::input::{TextInput, TextInputState, TextInputStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;
use std::ops::Range;

#[derive(Debug, Default)]
pub struct TextInputF<'a> {
    widget: TextInput<'a>,
}

#[derive(Debug, Clone)]
pub struct TextInputFState {
    pub widget: TextInputState,
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> TextInputF<'a> {
    /// Set the combined style.
    pub fn styles(mut self, style: TextInputStyle) -> Self {
        self.widget = self.widget.styles(style);
        self
    }

    /// Base text style.
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
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

// impl<'a> StatefulWidgetRef for TextInputF<'a> {
//     type State = TextInputFState;
//
//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         let widget = self.widget.clone().focused(state.is_focused());
//         widget.render(area, buf, &mut state.widget);
//     }
// }

impl<'a> StatefulWidget for TextInputF<'a> {
    type State = TextInputFState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let widget = self.widget.clone().focused(state.is_focused());
        widget.render(area, buf, &mut state.widget);
    }
}

impl Default for TextInputFState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl TextInputFState {
    /// Reset to empty.
    pub fn reset(&mut self) {
        self.widget.clear();
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

    /// Cursor position.
    pub fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.widget.screen_cursor()
    }

    /// Set text.
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.widget.set_value(s);
    }

    /// Text.
    pub fn value(&self) -> &str {
        self.widget.value()
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
    pub fn selected_value(&self) -> &str {
        self.widget.selected_value()
    }

    /// Set the cursor position from a visual position relative to the origin.
    pub fn set_screen_cursor(&mut self, rpos: isize, extend_selection: bool) {
        self.widget.set_screen_cursor(rpos, extend_selection);
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
    pub fn remove(&mut self, range: Range<usize>) {
        self.widget.value.remove(range);
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

impl HasFocusFlag for TextInputFState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, TextOutcome> for TextInputFState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> TextOutcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            TextOutcome::NotUsed
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for TextInputFState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
