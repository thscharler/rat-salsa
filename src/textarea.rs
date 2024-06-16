use crate::_private::NonExhaustive;
use crate::event::{FocusKeys, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_input::event::{ReadOnly, TextOutcome};
pub use rat_input::textarea::core;
use rat_input::textarea::core::{RopeGraphemes, TextRange};
use rat_scrolled::{ScrollingState, ScrollingWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidgetRef};
use ropey::{Rope, RopeSlice};

pub use rat_input::textarea::TextAreaStyle;

#[derive(Debug, Default, Clone)]
pub struct RTextArea<'a> {
    widget: rat_input::textarea::TextArea<'a>,
}

#[derive(Debug, Clone)]
pub struct RTextAreaState {
    pub widget: rat_input::textarea::TextAreaState,
    pub non_exhaustive: NonExhaustive,
}

impl<'a> RTextArea<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: TextAreaStyle) -> Self {
        self.widget = self.widget.styles(style);
        self
    }

    /// Base text style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style for selection
    #[inline]
    pub fn select_style(mut self, style: Style) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// List of text-styles.
    ///
    /// Use [rat_input::textarea::TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style<T: IntoIterator<Item = Style>>(mut self, styles: T) -> Self {
        self.widget = self.widget.text_style(styles);
        self
    }

    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> StatefulWidgetRef for RTextArea<'a> {
    type State = RTextAreaState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render_ref(area, buf, &mut state.widget)
    }
}

impl<'a> StatefulWidget for RTextArea<'a> {
    type State = RTextAreaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget)
    }
}

impl<'a> ScrollingWidget<RTextAreaState> for RTextArea<'a> {
    fn need_scroll(&self, area: Rect, state: &mut RTextAreaState) -> (bool, bool) {
        let sy = state.widget.line_len() > area.height as usize;
        (true, sy)
    }
}

impl Default for RTextAreaState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for RTextAreaState {
    fn focus(&self) -> &FocusFlag {
        &self.widget.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}

impl RTextAreaState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) -> bool {
        self.widget.clear()
    }

    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> (usize, usize) {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: (usize, usize)) -> bool {
        self.widget.set_offset(offset)
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> (usize, usize) {
        self.widget.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: (usize, usize), extend_selection: bool) -> bool {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> (usize, usize) {
        self.widget.anchor()
    }

    /// Text.
    #[inline]
    pub fn value(&self) -> String {
        self.widget.value()
    }

    /// Text value
    #[inline]
    pub fn value_range(&self, range: TextRange) -> Option<RopeSlice<'_>> {
        self.widget.value_range(range)
    }

    /// Text as Bytes iterator.
    #[inline]
    pub fn value_as_bytes(&self) -> ropey::iter::Bytes<'_> {
        self.widget.value_as_bytes()
    }

    /// Text as Btes iterator.
    #[inline]
    pub fn value_as_chars(&self) -> ropey::iter::Chars<'_> {
        self.widget.value_as_chars()
    }

    /// Set text.
    #[inline]
    pub fn set_value<S: AsRef<str>>(&mut self, s: S) {
        self.widget.set_value(s)
    }

    /// Set the text value as a Rope.
    /// Resets all internal state.
    #[inline]
    pub fn set_value_rope(&mut self, s: Rope) {
        self.widget.set_value_rope(s);
    }

    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Text length as grapheme count.
    #[inline]
    pub fn line_len(&self) -> usize {
        self.widget.line_len()
    }

    /// Line width as grapheme count.
    #[inline]
    pub fn line_width(&self, n: usize) -> Option<usize> {
        self.widget.line_width(n)
    }

    /// Grapheme iterator for a given line.
    /// This contains the \n at the end.
    #[inline]
    pub fn line(&self, n: usize) -> Option<RopeGraphemes<'_>> {
        self.widget.line(n)
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> TextRange {
        self.widget.selection()
    }

    /// Selection.
    #[inline]
    pub fn set_selection(&mut self, range: TextRange) -> bool {
        self.widget.set_selection(range)
    }

    /// Selection.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.widget.select_all()
    }

    /// Selection.
    #[inline]
    pub fn selected_value(&self) -> Option<RopeSlice<'_>> {
        self.widget.selected_value()
    }

    /// Clear all set styles.
    #[inline]
    pub fn clear_styles(&mut self) {
        self.widget.clear_styles();
    }

    /// Add a style for a [TextRange]. The style-nr refers to one
    /// of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: TextRange, style: usize) {
        self.widget.add_style(range, style);
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, pos: (usize, usize), result: &mut Vec<usize>) {
        self.widget.styles_at(pos, result)
    }

    /// Convert a byte position to a text area position.
    /// Uses grapheme based column indexes.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> Option<(usize, usize)> {
        self.widget.byte_pos(byte)
    }

    /// Convert a text area position to a byte range.
    /// Uses grapheme based column indexes.
    /// Returns (byte-start, byte-end) of the grapheme at the given position.
    #[inline]
    pub fn byte_at(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        self.widget.byte_at(pos)
    }

    /// Convert a char position to a text area position.
    /// Uses grapheme based column indexes.
    #[inline]
    pub fn char_pos(&self, byte: usize) -> Option<(usize, usize)> {
        self.widget.char_pos(byte)
    }

    /// Convert a text area position to a char position.
    /// Uses grapheme based column indexes.
    #[inline]
    pub fn char_at(&self, pos: (usize, usize)) -> Option<usize> {
        self.widget.char_at(pos)
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.widget.insert_char(c)
    }

    /// Insert a line break at the cursor position.
    #[inline]
    pub fn insert_newline(&mut self) -> bool {
        self.widget.insert_newline()
    }

    /// Deletes the given range.
    #[inline]
    pub fn delete_range(&mut self, range: TextRange) -> bool {
        self.widget.delete_range(range)
    }

    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        self.widget.delete_next_char()
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        self.widget.delete_prev_char()
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

    /// Move the cursor left. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_left(&mut self, n: usize, extend_selection: bool) -> bool {
        self.widget.move_left(n, extend_selection)
    }

    /// Move the cursor right. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_right(&mut self, n: usize, extend_selection: bool) -> bool {
        self.widget.move_right(n, extend_selection)
    }

    /// Move the cursor up. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_up(&mut self, n: usize, extend_selection: bool) -> bool {
        self.widget.move_up(n, extend_selection)
    }

    /// Move the cursor down. Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_down(&mut self, n: usize, extend_selection: bool) -> bool {
        self.widget.move_down(n, extend_selection)
    }

    /// Move the cursor to the start of the line.
    /// Scrolls the cursor to visible.
    /// Returns true if there was any real change.
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_start(extend_selection)
    }

    /// Move the cursor to the end of the line. Scrolls to visible, if
    /// necessary.
    /// Returns true if there was any real change.
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_end(extend_selection)
    }

    /// Move the cursor to the document start.
    pub fn move_to_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_start(extend_selection)
    }

    /// Move the cursor to the document end.
    pub fn move_to_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_end(extend_selection)
    }

    /// Move the cursor to the start of the visible area.
    pub fn move_to_screen_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_screen_start(extend_selection)
    }

    /// Move the cursor to the end of the visible area.
    pub fn move_to_screen_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_screen_end(extend_selection)
    }

    #[inline]
    pub fn move_to_next_word(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_next_word(extend_selection)
    }

    #[inline]
    pub fn move_to_prev_word(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_prev_word(extend_selection)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// Row is a row-index into the value, not a screen-row.
    /// x is the relative screen position.
    pub fn from_screen_col(&self, row: usize, x: usize) -> Option<usize> {
        self.widget.from_screen_col(row, x)
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn to_screen_col(&self, pos: (usize, usize)) -> Option<u16> {
        self.widget.to_screen_col(pos)
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

    /// Set the cursor position from a visual position relative to the origin.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: (i16, i16), extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(cursor, extend_selection)
    }
}

impl ScrollingState for RTextAreaState {
    fn vertical_max_offset(&self) -> usize {
        self.widget.vertical_max_offset()
    }

    fn vertical_offset(&self) -> usize {
        self.widget.vertical_offset()
    }

    fn vertical_page(&self) -> usize {
        self.widget.vertical_page()
    }

    fn horizontal_max_offset(&self) -> usize {
        self.widget.horizontal_max_offset()
    }

    fn horizontal_offset(&self) -> usize {
        self.widget.horizontal_offset()
    }

    fn horizontal_page(&self) -> usize {
        self.widget.horizontal_page()
    }

    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        self.widget.set_vertical_offset(offset)
    }

    fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        self.widget.set_horizontal_offset(offset)
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, TextOutcome> for RTextAreaState {
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

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for RTextAreaState {
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

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for RTextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        if self.gained_focus() {
            TextOutcome::NotUsed
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}
