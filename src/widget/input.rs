//!
//! Text input widget.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection.
//! * Scrolls with the cursor.
//! * Can set the cursor or use its own block cursor.
//! * Can show an indicator for invalid input.

use crate::widget::basic::ClearStyle;
use crate::{grapheme, ControlUI, ValidFlag};
use crate::{DefaultKeys, FrameWidget, HandleCrossterm, Input, MouseOnly};
use crate::{FocusFlag, HasFocusFlag, HasValidFlag};
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
    pub without_focus: bool,
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
            without_focus: false,
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

    /// Do accept keyboard events event without being focused.
    /// Useful for a catch field, eg "find stuff"
    pub fn without_focus(mut self, without_focus: bool) -> Self {
        self.without_focus = without_focus;
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
        if self.without_focus || focus {
            self.select_style
        } else {
            self.style
        }
    }
}

impl FrameWidget for TextInput {
    type State = TextInputState;

    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        state.without_focus = self.without_focus;

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
    /// Work without focus for key input.
    pub without_focus: bool,
    /// Area
    pub area: Rect,
    /// Mouse selection in progress.
    pub mouse_select: bool,
    /// Editing core
    pub value: core::InputCore,
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for TextInputState {
    #[allow(non_snake_case)]
    fn handle(&mut self, event: &crossterm::event::Event, _: DefaultKeys) -> ControlUI<A, E> {
        use crossterm::event::KeyCode::*;
        use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyModifiers};

        const NONE: KeyModifiers = KeyModifiers::NONE;
        const CTRL: KeyModifiers = KeyModifiers::CONTROL;
        const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
        let CTRL_SHIFT: KeyModifiers = KeyModifiers::SHIFT | KeyModifiers::CONTROL;

        let req = match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => 'f: {
                if !self.focus.get() && !self.without_focus {
                    break 'f None;
                }

                match (*code, *modifiers) {
                    (Left, NONE) => Some(InputRequest::GoToPrevChar(false)),
                    (Right, NONE) => Some(InputRequest::GoToNextChar(false)),
                    (Left, CTRL) => Some(InputRequest::GoToPrevWord(false)),
                    (Right, CTRL) => Some(InputRequest::GoToNextWord(false)),
                    (Home, NONE) => Some(InputRequest::GoToStart(false)),
                    (End, NONE) => Some(InputRequest::GoToEnd(false)),

                    (Left, SHIFT) => Some(InputRequest::GoToPrevChar(true)),
                    (Right, SHIFT) => Some(InputRequest::GoToNextChar(true)),
                    (Left, m) if m == CTRL_SHIFT => Some(InputRequest::GoToPrevWord(true)),
                    (Right, m) if m == CTRL_SHIFT => Some(InputRequest::GoToNextWord(true)),
                    (Home, SHIFT) => Some(InputRequest::GoToStart(true)),
                    (End, SHIFT) => Some(InputRequest::GoToEnd(true)),

                    (Char('a'), CTRL) => Some(InputRequest::SelectAll),

                    (Backspace, NONE) => Some(InputRequest::DeletePrevChar),
                    (Delete, NONE) => Some(InputRequest::DeleteNextChar),

                    (Backspace, CTRL) => Some(InputRequest::DeletePrevWord),
                    (Delete, CTRL) => Some(InputRequest::DeleteNextWord),

                    (Char('d'), CTRL) => Some(InputRequest::DeleteLine),
                    (Backspace, m) if m == CTRL_SHIFT => Some(InputRequest::DeleteTillStart),
                    (Delete, m) if m == CTRL_SHIFT => Some(InputRequest::DeleteTillEnd),

                    (Char(c), NONE) => Some(InputRequest::InsertChar(c)),
                    (Char(c), SHIFT) => Some(InputRequest::InsertChar(c)),
                    (_, _) => None,
                }
            }
            _ => return self.handle(event, MouseOnly),
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TextInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> ControlUI<A, E> {
        use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let req = match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.mouse_select = true;
                    let c = column - self.area.x;
                    Some(InputRequest::SetCursor(c as isize, false))
                } else {
                    None
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if self.mouse_select {
                    self.mouse_select = false;
                }
                None
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if self.mouse_select {
                    let c =
                        (*column as isize) - (self.area.x as isize)
                     ;
                    Some(InputRequest::SetCursor(c , true))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

/// Mapping from events to abstract editing requests.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum InputRequest {
    /// Set the cursor. This is the *visible* position relative to the
    /// offset. It may be negative too.
    SetCursor(isize, bool),
    Select(usize, usize),
    InsertChar(char),
    GoToPrevChar(bool),
    GoToNextChar(bool),
    GoToPrevWord(bool),
    GoToNextWord(bool),
    GoToStart(bool),
    GoToEnd(bool),
    SelectAll,
    DeletePrevChar,
    DeleteNextChar,
    DeletePrevWord,
    DeleteNextWord,
    DeleteLine,
    DeleteTillStart,
    DeleteTillEnd,
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
    pub fn set_cursor(&mut self, cursor: usize) {
        self.value.set_cursor(cursor, false);
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
        self.value.is_anchored()
    }

    /// Selection.
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.value.set_cursor(cursor, false);
        self.value.set_cursor(anchor, true);
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

    /// Extracts the visible part.
    pub fn visible_range(&self) -> Range<usize> {
        let len = min(self.value.offset() + self.value.width(), self.value.len());
        self.value.offset()..len
    }

    /// Extracts the visible selection.
    pub fn visible_selection(&self) -> Range<usize> {
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
    pub fn visible_part(&mut self) -> (&str, &str, &str, &str, &str) {
        grapheme::split5(
            self.value.as_str(),
            self.cursor(),
            self.visible_range(),
            self.visible_selection(),
        )
    }

    /// Visible cursor position.
    pub fn visible_cursor(&mut self) -> u16 {
        (self.value.cursor() - self.value.offset()) as u16
    }
}

impl<A, E> Input<ControlUI<A, E>> for TextInputState {
    type Request = InputRequest;

    fn perform(&mut self, action: Self::Request) -> ControlUI<A, E> {
        use InputRequest::*;

        match action {
            SetCursor(rpos, anchor) => {
                let pos = if rpos < 0 {
                    self.value.offset().saturating_sub(-rpos as usize)
                } else {
                    self.value.offset() + rpos as usize
                };
                if self.value.cursor() == pos {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(pos, anchor);
                    ControlUI::Change
                }
            }
            Select(anchor, cursor) => {
                self.value.set_cursor(anchor, false);
                self.value.set_cursor(cursor, true);
                ControlUI::Change
            }
            InsertChar(c) => {
                self.value.insert_char(c);
                ControlUI::Change
            }
            DeletePrevChar => {
                if self.value.is_anchored() {
                    self.value.replace(self.value.selection(), "");
                    ControlUI::Change
                } else if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value
                        .replace(self.value.cursor() - 1..self.value.cursor(), "");
                    ControlUI::Change
                }
            }
            DeleteNextChar => {
                if self.value.is_anchored() {
                    self.value.replace(self.value.selection(), "");
                    ControlUI::Change
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value
                        .replace(self.value.cursor()..self.value.cursor() + 1, "");
                    ControlUI::Change
                }
            }
            GoToPrevChar(anchor) => {
                if !anchor && self.value.is_anchored() {
                    let c = self.value.selection().start;
                    self.value.set_cursor(c, false);
                    ControlUI::Change
                } else if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(self.value.cursor() - 1, anchor);
                    ControlUI::Change
                }
            }
            GoToNextChar(anchor) => {
                if !anchor && self.value.is_anchored() {
                    let c = self.value.selection().end;
                    self.value.set_cursor(c, false);
                    ControlUI::Change
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(self.value.cursor() + 1, anchor);
                    ControlUI::Change
                }
            }
            GoToPrevWord(anchor) => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    let cursor = self.value.prev_word_boundary();
                    self.value.set_cursor(cursor, anchor);
                    ControlUI::Change
                }
            }
            GoToNextWord(anchor) => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    let cursor = self.value.next_word_boundary();
                    self.value.set_cursor(cursor, anchor);
                    ControlUI::Change
                }
            }
            DeleteLine => {
                if self.value.is_empty() {
                    ControlUI::NoChange
                } else {
                    self.value.set_value("");
                    ControlUI::Change
                }
            }
            DeletePrevWord => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    let prev = self.value.prev_word_boundary();
                    self.value.replace(prev..self.value.cursor(), "");
                    ControlUI::Change
                }
            }
            DeleteNextWord => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    let next = self.value.next_word_boundary();
                    self.value.replace(self.value.cursor()..next, "");
                    ControlUI::Change
                }
            }
            GoToStart(anchor) => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(0, anchor);
                    ControlUI::Change
                }
            }
            GoToEnd(anchor) => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(self.value.len(), anchor);
                    ControlUI::Change
                }
            }
            DeleteTillEnd => {
                self.value
                    .replace(self.value.cursor()..self.value.len(), "");
                ControlUI::Change
            }
            DeleteTillStart => {
                self.value.replace(0..self.value.cursor(), "");
                ControlUI::Change
            }
            SelectAll => {
                self.value.set_cursor(0, false);
                self.value.set_cursor(self.value.len(), true);
                ControlUI::Change
            }
        }
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
        pub fn set_cursor(&mut self, cursor: usize, anchor: bool) {
            let cursor = if cursor > self.len { self.len } else { cursor };

            self.cursor = cursor;

            if !anchor {
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
        pub fn is_anchored(&self) -> bool {
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
