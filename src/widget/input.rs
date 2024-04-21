//!
//! Text input widget.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection.
//! * Scrolls with the cursor.
//! * Can set the cursor or use its own block cursor.
//! * Can show an indicator for invalid input.

use crate::widget::basic::ClearStyle;
use crate::{ct_event, grapheme, ControlUI, ValidFlag};
use crate::{DefaultKeys, FrameWidget, HandleCrossterm, MouseOnly};
use crate::{FocusFlag, HasFocusFlag, HasValidFlag};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::Frame;
use std::cmp::min;
use std::ops::Range;

/// Text input widget.
#[derive(Debug)]
pub struct TextInput {
    pub terminal_cursor: bool,
    pub insets: Margin,
    pub style: Style,
    pub focus_style: Style,
    pub select_style: Style,
    pub cursor_style: Option<Style>,
    pub invalid_style: Option<Style>,
    pub invalid_char: char,
}

/// Combined style for the widget.
#[derive(Debug, Default)]
pub struct TextInputStyle {
    pub style: Style,
    pub focus: Style,
    pub select: Style,
    pub cursor: Option<Style>,
    pub invalid: Option<Style>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            terminal_cursor: true,
            insets: Default::default(),
            style: Default::default(),
            focus_style: Default::default(),
            select_style: Default::default(),
            cursor_style: None,
            invalid_style: None,
            invalid_char: '⁉',
        }
    }
}

impl TextInput {
    /// Use extra insets for the text input.
    pub fn insets(mut self, insets: Margin) -> Self {
        self.insets = insets;
        self
    }

    /// Use our own cursor indicator or the terminal cursor.
    pub fn terminal_cursor(mut self, terminal: bool) -> Self {
        self.terminal_cursor = terminal;
        self
    }

    /// Set the combined style.
    pub fn style(mut self, style: TextInputStyle) -> Self {
        self.style = style.style;
        self.focus_style = style.focus;
        self.select_style = style.select;
        self.cursor_style = style.cursor;
        self.invalid_style = style.invalid;
        self
    }

    /// Base text style.
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = style.into();
        self
    }

    /// Style for our own cursor.
    pub fn cursor_style(mut self, style: impl Into<Style>) -> Self {
        self.cursor_style = Some(style.into());
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.invalid_style = Some(style.into());
        self
    }

    /// Marker character for invalid field.
    pub fn invalid_char(mut self, invalid: char) -> Self {
        self.invalid_char = invalid;
        self
    }

    // focused or base
    fn active_style(&self, focus: bool) -> Style {
        if focus {
            self.focus_style
        } else {
            self.style
        }
    }

    // focused or base
    fn active_select_style(&self, focus: bool) -> Style {
        if focus {
            self.select_style
        } else {
            self.style
        }
    }
}

impl FrameWidget for TextInput {
    type State = TextInputState;

    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        let mut l_area = area.inner(&self.insets);
        let l_invalid = if !state.valid.get() {
            l_area.width -= 1;
            Rect::new(l_area.x + l_area.width, l_area.y, 1, 1)
        } else {
            Rect::new(l_area.x + l_area.width, l_area.y, 0, 1)
        };

        state.area = l_area;
        state.value.set_width(state.area.width as usize);

        let focus = state.focus.get();
        let l_input = state.area;

        let (before, cursor1, select, cursor2, after) = state.visible_part();

        let mut spans = Vec::new();
        if !before.is_empty() {
            spans.push(Span::styled(before, self.active_style(focus)));
        }
        if !cursor1.is_empty() {
            if let Some(cursor_style) = self.cursor_style {
                spans.push(Span::styled(cursor1, cursor_style));
            } else {
                spans.push(Span::styled(cursor1, self.active_select_style(focus)));
            }
        }
        if !select.is_empty() {
            spans.push(Span::styled(select, self.active_select_style(focus)));
        }
        if !cursor2.is_empty() {
            if let Some(cursor_style) = self.cursor_style {
                spans.push(Span::styled(cursor2, cursor_style));
            } else {
                spans.push(Span::styled(cursor2, self.active_style(focus)));
            }
        }
        if !after.is_empty() {
            spans.push(Span::styled(after, self.active_style(focus)));
        }

        let line = Line::from(spans);
        let clear = ClearStyle::default().style(self.active_style(focus));

        frame.render_widget(clear, area);
        frame.render_widget(line, l_input);
        if !state.valid.get() {
            let style = if let Some(style) = self.invalid_style {
                style
            } else {
                self.active_style(focus)
            };

            let invalid = Span::from(self.invalid_char.to_string()).style(style);
            frame.render_widget(invalid, l_invalid);
        }
        if self.terminal_cursor && focus {
            frame.set_cursor(l_input.x + state.visible_cursor(), l_input.y);
        }
    }
}

/// Input state data.
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    /// Focus
    pub focus: FocusFlag,
    /// Valid.
    pub valid: ValidFlag,
    /// Area
    pub area: Rect,
    /// Mouse selection in progress.
    pub mouse_select: bool,
    /// Editing core
    pub value: core::InputCore,
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for TextInputState {
    #[allow(non_snake_case)]
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = 'f: {
            if self.is_focused() {
                match event {
                    ct_event!(keycode press Left) => self.move_to_prev(false),
                    ct_event!(keycode press Right) => self.move_to_next(false),
                    ct_event!(keycode press CONTROL-Left) => {
                        let pos = self.prev_word_boundary();
                        self.set_cursor(pos, false);
                    }
                    ct_event!(keycode press CONTROL-Right) => {
                        let pos = self.next_word_boundary();
                        self.set_cursor(pos, false);
                    }
                    ct_event!(keycode press Home) => self.set_cursor(0, false),
                    ct_event!(keycode press End) => self.set_cursor(self.len(), false),
                    ct_event!(keycode press SHIFT-Left) => self.move_to_prev(true),
                    ct_event!(keycode press SHIFT-Right) => self.move_to_next(true),
                    ct_event!(keycode press CONTROL_SHIFT-Left) => {
                        let pos = self.prev_word_boundary();
                        self.set_cursor(pos, true);
                    }
                    ct_event!(keycode press CONTROL_SHIFT-Right) => {
                        let pos = self.next_word_boundary();
                        self.set_cursor(pos, true);
                    }
                    ct_event!(keycode press SHIFT-Home) => self.set_cursor(0, true),
                    ct_event!(keycode press SHIFT-End) => self.set_cursor(self.len(), true),
                    ct_event!(key press CONTROL-'a') => self.set_selection(0, self.len()),
                    ct_event!(keycode press Backspace) => self.delete_prev_char(),
                    ct_event!(keycode press Delete) => self.delete_next_char(),
                    ct_event!(keycode press CONTROL-Backspace) => {
                        let prev = self.prev_word_boundary();
                        self.replace(prev..self.cursor(), "");
                    }
                    ct_event!(keycode press CONTROL-Delete) => {
                        let next = self.next_word_boundary();
                        self.replace(self.cursor()..next, "");
                    }
                    ct_event!(key press CONTROL-'d') => self.set_value(""),
                    ct_event!(keycode press CONTROL_SHIFT-Backspace) => {
                        self.replace(0..self.cursor(), "")
                    }
                    ct_event!(keycode press CONTROL_SHIFT-Delete) => {
                        self.replace(self.cursor()..self.len(), "")
                    }
                    ct_event!(key press c) | ct_event!(key press SHIFT-c) => self.insert_char(*c),
                    _ => break 'f ControlUI::Continue,
                }
                ControlUI::Change
            } else {
                ControlUI::Continue
            }
        };

        res.or_else(|| self.handle(event, MouseOnly))
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TextInputState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let res = match event {
            ct_event!(mouse down Left for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.mouse_select = true;
                    let c = column - self.area.x;
                    self.set_offset_relative_cursor(c as isize, false);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse drag Left for column, _row) => {
                if self.mouse_select {
                    let c = (*column as isize) - (self.area.x as isize);
                    self.set_offset_relative_cursor(c, true);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse moved) => {
                self.mouse_select = false;
                ControlUI::Continue
            }
            _ => ControlUI::Continue,
        };

        res
    }
}

impl TextInputState {
    /// Reset to empty.
    pub fn reset(&mut self) {
        self.value.clear();
    }

    /// Offset shown.
    pub fn offset(&self) -> usize {
        self.value.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    pub fn set_offset(&mut self, offset: usize) {
        self.value.set_offset(offset);
    }

    /// Display width.
    pub fn width(&self) -> usize {
        self.value.width()
    }

    /// Display width
    pub fn set_width(&mut self, width: usize) {
        self.value.set_width(width);
    }

    /// Set the cursor position, reset selection.
    pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) {
        self.value.set_cursor(cursor, extend_selection);
    }

    /// Cursor position.
    pub fn cursor(&self) -> usize {
        self.value.cursor()
    }

    /// Set text.
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.value.set_value(s);
    }

    /// Text.
    pub fn value(&self) -> &str {
        self.value.value()
    }

    /// Text
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    /// Empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Text length as grapheme count.
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Selection.
    pub fn has_selection(&self) -> bool {
        self.value.has_selection()
    }

    /// Selection.
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.value.set_cursor(anchor, false);
        self.value.set_cursor(cursor, true);
    }

    /// Selection.
    pub fn select_all(&mut self) {
        self.value.set_cursor(0, false);
        self.value.set_cursor(self.value.len(), true);
    }

    /// Selection.
    pub fn selection(&self) -> Range<usize> {
        self.value.selection()
    }

    /// Selection.
    pub fn selection_str(&self) -> &str {
        grapheme::split3(self.value.as_str(), self.value.selection()).1
    }

    /// Previous word boundary
    pub fn prev_word_boundary(&self) -> usize {
        self.value.prev_word_boundary()
    }

    /// Next word boundary
    pub fn next_word_boundary(&self) -> usize {
        self.value.next_word_boundary()
    }

    /// Set the cursor position from a visual position relative to the origin.
    pub fn set_offset_relative_cursor(&mut self, rpos: isize, extend_selection: bool) {
        let pos = if rpos < 0 {
            self.value.offset().saturating_sub(-rpos as usize)
        } else {
            self.value.offset() + rpos as usize
        };
        self.value.set_cursor(pos, extend_selection);
    }

    /// Move to the next char.
    pub fn move_to_next(&mut self, extend_selection: bool) {
        if !extend_selection && self.value.has_selection() {
            let c = self.value.selection().end;
            self.value.set_cursor(c, false);
        } else if self.value.cursor() < self.value.len() {
            self.value
                .set_cursor(self.value.cursor() + 1, extend_selection);
        }
    }

    /// Move to the previous char.
    pub fn move_to_prev(&mut self, extend_selection: bool) {
        if !extend_selection && self.value.has_selection() {
            let c = self.value.selection().start;
            self.value.set_cursor(c, false);
        } else if self.value.cursor() > 0 {
            self.value
                .set_cursor(self.value.cursor() - 1, extend_selection);
        }
    }

    /// Insert a char a the current position.
    pub fn insert_char(&mut self, c: char) {
        self.value.insert_char(c);
    }

    /// Replace the given range with a new string.
    pub fn replace(&mut self, range: Range<usize>, new: &str) {
        self.value.replace(range, new);
    }

    /// Delete the char before the cursor.
    pub fn delete_prev_char(&mut self) {
        if self.value.has_selection() {
            self.value.replace(self.value.selection(), "");
        } else if self.value.cursor() == 0 {
        } else {
            self.value
                .replace(self.value.cursor() - 1..self.value.cursor(), "");
        }
    }

    /// Delete the char after the cursor.
    pub fn delete_next_char(&mut self) {
        if self.value.has_selection() {
            self.value.replace(self.value.selection(), "");
        } else if self.value.cursor() == self.value.len() {
        } else {
            self.value
                .replace(self.value.cursor()..self.value.cursor() + 1, "");
        }
    }

    /// Extracts the visible part.
    fn visible_range(&self) -> Range<usize> {
        let len = min(self.value.offset() + self.value.width(), self.value.len());
        self.value.offset()..len
    }

    /// Extracts the visible selection.
    fn visible_selection(&self) -> Range<usize> {
        let width = self.value.width();
        let offset = self.value.offset();
        let Range { mut start, mut end } = self.value.selection();

        if start < offset {
            start = offset;
        } else if start > offset + width {
            start = offset + width;
        }
        if end < offset {
            end = offset;
        } else if end > offset + width {
            end = offset + width;
        }

        start..end
    }

    /// Extracts the visible parts. The result is (before, cursor1, selection, cursor2, after).
    /// One cursor1 and cursor2 is an empty string.
    fn visible_part(&mut self) -> (&str, &str, &str, &str, &str) {
        grapheme::split5(
            self.value.as_str(),
            self.cursor(),
            self.visible_range(),
            self.visible_selection(),
        )
    }

    /// Visible cursor position.
    fn visible_cursor(&mut self) -> u16 {
        (self.value.cursor() - self.value.offset()) as u16
    }
}

impl HasFocusFlag for TextInputState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasValidFlag for TextInputState {
    fn valid(&self) -> &ValidFlag {
        &self.valid
    }
}

pub mod core {
    use crate::grapheme;
    #[allow(unused_imports)]
    use log::debug;
    use std::mem;
    use std::ops::Range;
    use unicode_segmentation::UnicodeSegmentation;

    /// Text editing core.
    #[derive(Debug, Default, Clone)]
    pub struct InputCore {
        // Text
        value: String,
        // Len in grapheme count.
        len: usize,

        offset: usize,
        width: usize,

        cursor: usize,
        anchor: usize,

        char_buf: String,
        buf: String,
    }

    impl InputCore {
        /// Offset
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Change the offset
        pub fn set_offset(&mut self, offset: usize) {
            if offset > self.len {
                self.offset = self.len;
            } else if offset > self.cursor {
                self.offset = self.cursor;
            } else if offset + self.width < self.cursor {
                self.offset = self.cursor - self.width;
            } else {
                self.offset = offset;
            }
        }

        /// Display width
        pub fn width(&self) -> usize {
            self.width
        }

        /// Display width
        pub fn set_width(&mut self, width: usize) {
            self.width = width;

            if self.offset + width < self.cursor {
                self.offset = self.cursor - self.width;
            }
        }

        /// Cursor position as grapheme-idx. Moves the cursor to the new position,
        /// but can leave the current cursor position as anchor of the selection.
        pub fn set_cursor(&mut self, cursor: usize, extend_selection: bool) {
            let cursor = if cursor > self.len { self.len } else { cursor };

            self.cursor = cursor;

            if !extend_selection {
                self.anchor = cursor;
            }

            if self.offset > cursor {
                self.offset = cursor;
            } else if self.offset + self.width < cursor {
                self.offset = cursor - self.width;
            }
        }

        /// Cursor position as grapheme-idx.
        pub fn cursor(&self) -> usize {
            self.cursor
        }

        /// Set the value. Resets cursor and anchor to 0.
        pub fn set_value<S: Into<String>>(&mut self, s: S) {
            self.value = s.into();
            self.len = self.value.graphemes(true).count();
            self.cursor = 0;
            self.offset = 0;
            self.anchor = 0;
        }

        /// Value
        pub fn value(&self) -> &str {
            self.value.as_str()
        }

        /// Value
        pub fn as_str(&self) -> &str {
            self.value.as_str()
        }

        /// Clear
        pub fn clear(&mut self) {
            self.set_value("");
        }

        /// Empty
        pub fn is_empty(&self) -> bool {
            self.value.is_empty()
        }

        /// Value lenght as grapheme-count
        pub fn len(&self) -> usize {
            self.len
        }

        /// Selection anchor
        pub fn anchor(&self) -> usize {
            self.anchor
        }

        /// Anchor is active
        pub fn has_selection(&self) -> bool {
            self.anchor != self.cursor
        }

        /// Selection.
        pub fn selection(&self) -> Range<usize> {
            if self.cursor < self.anchor {
                self.cursor..self.anchor
            } else {
                self.anchor..self.cursor
            }
        }

        /// Find next word.
        pub fn next_word_boundary(&self) -> usize {
            if self.cursor == self.len {
                self.len
            } else {
                self.value
                    .graphemes(true)
                    .enumerate()
                    .skip(self.cursor)
                    .skip_while(|(_, c)| grapheme::is_alphanumeric(c))
                    .find(|(_, c)| grapheme::is_alphanumeric(c))
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| self.len)
            }
        }

        /// Find previous word.
        pub fn prev_word_boundary(&self) -> usize {
            if self.cursor == 0 {
                0
            } else {
                self.value
                    .graphemes(true)
                    .rev()
                    .skip(self.len - self.cursor)
                    .skip_while(|c| !grapheme::is_alphanumeric(c))
                    .skip_while(|c| grapheme::is_alphanumeric(c))
                    .count()
            }
        }

        /// Insert a char, replacing the selection.
        pub fn insert_char(&mut self, new: char) {
            let selection = self.selection();

            let mut char_buf = mem::take(&mut self.char_buf);
            char_buf.clear();
            char_buf.push(new);
            self.replace(selection, char_buf.as_str());
            self.char_buf = char_buf;
        }

        /// Insert a string, replacing the selection.
        pub fn replace(&mut self, range: Range<usize>, new: &str) {
            let new_len = new.graphemes(true).count();

            let (before_str, sel_str, after_str) = grapheme::split3(self.value.as_str(), range);
            let sel_len = sel_str.graphemes(true).count();
            let before_len = before_str.graphemes(true).count();

            self.len -= sel_len;
            self.len += new_len;

            if self.cursor >= before_len + sel_len {
                self.cursor -= sel_len;
                self.cursor += new_len;
            } else if self.cursor >= before_len {
                self.cursor = before_len + new_len;
            }

            if self.anchor >= before_len + sel_len {
                self.anchor -= sel_len;
                self.anchor += new_len;
            } else if self.anchor >= before_len {
                self.anchor = before_len + new_len;
            }

            // fix offset
            if self.offset > self.cursor {
                self.offset = self.cursor;
            } else if self.offset + self.width < self.cursor {
                self.offset = self.cursor - self.width;
            }

            self.buf.clear();
            self.buf.push_str(before_str);
            self.buf.push_str(new);
            self.buf.push_str(after_str);

            mem::swap(&mut self.value, &mut self.buf);
        }
    }
}
