//! Text input with an input mask.
//!
//! * 0: must enter digit, display as 0
//! * 9: can enter digit, display as space
//! * H: must enter a hex digit, display as 0
//! * h: can enter a hex digit, display as space
//! * O: must enter an octal digit, display as 0
//! * o: can enter an octal digit, display as space
//! * L: must enter letter, display as X
//! * l: can enter letter, display as space
//! * A: must enter letter or digit, display as X
//! * a: can enter letter or digit, display as space
//! * C: must enter character or space, display as space
//! * c: can enter character or space, display as space
//! * _: anything, display as space
//! * #: digit, plus or minus sign, display as space
//! * . , : ; - /: grouping characters move the cursor when entered
//!
//! Inspired by https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da

use crate::basic::ClearStyle;
use crate::focus::FocusFlag;
use crate::mask_input::core::{split3, split5, CursorPos};
use crate::{ControlUI, HandleEvent, WidgetExt};
use crossterm::event::KeyCode::{Backspace, Char, Delete, End, Home, Left, Right};
use crossterm::event::{
    Event, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
#[allow(unused_imports)]
use log::debug;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::Frame;
use std::cmp::min;
use std::ops::Range;

#[derive(Debug, Default)]
pub struct InputMask {
    pub without_focus: bool,
    pub insets: Margin,
    pub style: Style,
    pub focus_style: Style,
    pub select_style: Style,
    pub cursor_style: Option<Style>,
    pub invalid_style: Option<Style>,
}

#[derive(Debug, Default)]
pub struct InputMaskStyle {
    pub style: Style,
    pub focus: Style,
    pub select: Style,
    pub cursor: Option<Style>,
    pub invalid: Option<Style>,
}

impl InputMask {
    pub fn insets(mut self, insets: Margin) -> Self {
        self.insets = insets;
        self
    }

    pub fn without_focus(mut self, without_focus: bool) -> Self {
        self.without_focus = without_focus;
        self
    }

    pub fn style(mut self, style: InputMaskStyle) -> Self {
        self.style = style.style;
        self.focus_style = style.focus;
        self.select_style = style.select;
        self.cursor_style = style.cursor;
        self.invalid_style = style.invalid;
        self
    }

    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }

    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = style.into();
        self
    }

    pub fn cursor_style(mut self, style: impl Into<Style>) -> Self {
        self.cursor_style = Some(style.into());
        self
    }

    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.invalid_style = Some(style.into());
        self
    }

    fn active_style(&self, focus: bool) -> Style {
        if focus {
            self.focus_style
        } else {
            self.style
        }
    }

    fn active_select_style(&self, focus: bool) -> Style {
        if self.without_focus || focus {
            self.select_style
        } else {
            self.style
        }
    }
}

impl WidgetExt for InputMask {
    type State = InputMaskState;

    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        state.without_focus = self.without_focus;

        let mut l_area = area.inner(&self.insets);
        let l_invalid = if state.focus.is_invalid() {
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
        if before.len() > 0 {
            spans.push(Span::styled(before, self.active_style(focus)));
        }
        if cursor1.len() > 0 {
            if let Some(cursor_style) = self.cursor_style.clone() {
                spans.push(Span::styled(cursor1, cursor_style));
            } else {
                spans.push(Span::styled(cursor1, self.active_select_style(focus)));
            }
        }
        if select.len() > 0 {
            spans.push(Span::styled(select, self.active_select_style(focus)));
        }
        if cursor2.len() > 0 {
            if let Some(cursor_style) = self.cursor_style.clone() {
                spans.push(Span::styled(cursor2, cursor_style));
            } else {
                spans.push(Span::styled(cursor2, self.active_style(focus)));
            }
        }
        if after.len() > 0 {
            spans.push(Span::styled(after, self.active_style(focus)));
        }

        let line = Line::from(spans);
        let clear = ClearStyle::default().style(self.active_style(focus));

        frame.render_widget(clear, area);
        frame.render_widget(line, l_input);
        if state.focus.is_invalid() {
            let style = if let Some(style) = self.invalid_style {
                style
            } else {
                self.active_style(focus)
            };

            let invalid = Span::from("⁉").style(style);
            frame.render_widget(invalid, l_invalid);
        }
        if focus {
            frame.set_cursor(l_input.x + state.visible_cursor(), l_input.y);
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputMaskState {
    pub focus: FocusFlag,
    /// Work without focus for key input.
    pub without_focus: bool,
    pub area: Rect,
    pub mouse_select: bool,
    pub value: core::InputMaskCore,
}

impl<A, E> HandleEvent<A, E> for InputMaskState {
    fn handle(&mut self, evt: &Event) -> ControlUI<A, E> {
        #[allow(non_snake_case)]
        let CONTROL_SHIFT = KeyModifiers::SHIFT | KeyModifiers::CONTROL;

        let req = match evt {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.without_focus || self.focus.get() {
                    match (*code, *modifiers) {
                        (Left, KeyModifiers::NONE) => Some(InputRequest::GoToPrevChar(false)),
                        (Right, KeyModifiers::NONE) => Some(InputRequest::GoToNextChar(false)),
                        (Left, KeyModifiers::CONTROL) => Some(InputRequest::GoToPrevWord(false)),
                        (Right, KeyModifiers::CONTROL) => Some(InputRequest::GoToNextWord(false)),
                        (Home, KeyModifiers::NONE) => Some(InputRequest::GoToStart(false)),
                        (End, KeyModifiers::NONE) => Some(InputRequest::GoToEnd(false)),

                        (Left, KeyModifiers::SHIFT) => Some(InputRequest::GoToPrevChar(true)),
                        (Right, KeyModifiers::SHIFT) => Some(InputRequest::GoToNextChar(true)),
                        (Left, m) if m == CONTROL_SHIFT => Some(InputRequest::GoToPrevWord(true)),
                        (Right, m) if m == CONTROL_SHIFT => Some(InputRequest::GoToNextWord(true)),
                        (Home, KeyModifiers::SHIFT) => Some(InputRequest::GoToStart(true)),
                        (End, KeyModifiers::SHIFT) => Some(InputRequest::GoToEnd(true)),

                        (Char('a'), KeyModifiers::CONTROL) => Some(InputRequest::SelectAll),

                        (Backspace, KeyModifiers::NONE) => Some(InputRequest::DeletePrevChar),
                        (Delete, KeyModifiers::NONE) => Some(InputRequest::DeleteNextChar),

                        (Backspace, KeyModifiers::CONTROL) => Some(InputRequest::DeletePrevWord),
                        (Delete, KeyModifiers::CONTROL) => Some(InputRequest::DeleteNextWord),

                        (Char('d'), KeyModifiers::CONTROL) => Some(InputRequest::DeleteLine),
                        (Backspace, m) if m == CONTROL_SHIFT => Some(InputRequest::DeleteTillStart),
                        (Delete, m) if m == CONTROL_SHIFT => Some(InputRequest::DeleteTillEnd),

                        (Char(c), KeyModifiers::NONE) => Some(InputRequest::InsertChar(c)),
                        (Char(c), KeyModifiers::SHIFT) => Some(InputRequest::InsertChar(c)),
                        (_, _) => None,
                    }
                } else {
                    None
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.mouse_select = true;
                    let c = column - self.area.x;
                    Some(InputRequest::SetCursor(c as usize, false))
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
                    let c = if *column >= self.area.x {
                        column - self.area.x
                    } else {
                        0
                    };
                    Some(InputRequest::SetCursor(c as usize, true))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(req) = req {
            self.handle_request(req)
        } else {
            ControlUI::Continue
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum InputRequest {
    SetCursor(usize, bool),
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

impl InputMaskState {
    pub fn reset(&mut self) {
        self.value.clear();
    }

    pub fn offset(&self) -> usize {
        self.value.offset()
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.value.set_offset(offset);
    }

    pub fn width(&self) -> usize {
        self.value.width()
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.value.set_cursor(cursor, false);
    }

    pub fn cursor(&self) -> usize {
        self.value.cursor()
    }

    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.value.set_display_mask(s);
    }

    pub fn display_mask(&self) -> &str {
        self.value.display_mask()
    }

    pub fn set_mask<S: Into<String>>(&mut self, s: S) {
        self.value.set_mask(s);
    }

    pub fn mask(&self) -> &str {
        self.value.mask()
    }

    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.value.set_value(s);
    }

    pub fn value(&self) -> &str {
        self.value.value()
    }

    /// Value with optional spaces removed.
    pub fn compact_value(&self) -> String {
        self.value.compact_value()
    }

    /// Value that has been prepared for rendering
    pub fn render_str(&self) -> &str {
        self.value.render_str()
    }

    pub fn as_str(&self) -> &str {
        self.value.value()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn has_selection(&self) -> bool {
        self.value.is_anchored()
    }

    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.value.set_cursor(anchor, false);
        self.value.set_cursor(cursor, true);
    }

    pub fn select_all(&mut self) {
        // the other way round it fails if width is 0.
        self.value.set_cursor(self.value.len(), false);
        self.value.set_cursor(0, true);
    }

    pub fn selection(&self) -> Range<usize> {
        self.value.selection()
    }

    pub fn selection_str(&self) -> &str {
        split3(self.value.value(), self.value.selection()).1
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

    /// Extracts the visible part.
    pub fn visible_part(&mut self) -> (&str, &str, &str, &str, &str) {
        self.value.render_value();
        split5(
            self.value.render_str(),
            self.cursor(),
            self.visible_range(),
            self.visible_selection(),
        )
    }

    /// Visible cursor position.
    pub fn visible_cursor(&mut self) -> u16 {
        (self.value.cursor() - self.value.offset()) as u16
    }

    fn handle_request<A, E>(&mut self, req: InputRequest) -> ControlUI<A, E> {
        use InputRequest::*;

        match req {
            SetCursor(pos, anchor) => {
                let pos = pos + self.value.offset();
                if self.value.cursor() == pos {
                    ControlUI::Unchanged
                } else {
                    self.value.set_cursor(pos, anchor);
                    ControlUI::Changed
                }
            }
            Select(anchor, cursor) => {
                self.value.set_cursor(anchor, false);
                self.value.set_cursor(cursor, true);
                ControlUI::Changed
            }
            InsertChar(c) => {
                self.value.insert_char(c);
                ControlUI::Changed
            }
            DeletePrevChar => {
                if self.value.is_anchored() {
                    self.value.remove(self.value.selection(), CursorPos::Start);
                    ControlUI::Changed
                } else if self.value.cursor() == 0 {
                    ControlUI::Unchanged
                } else {
                    self.value.remove(
                        self.value.cursor() - 1..self.value.cursor(),
                        CursorPos::Start,
                    );
                    ControlUI::Changed
                }
            }
            DeleteNextChar => {
                if self.value.is_anchored() {
                    self.value.remove(self.value.selection(), CursorPos::End);
                    ControlUI::Changed
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::Unchanged
                } else {
                    self.value
                        .remove(self.value.cursor()..self.value.cursor() + 1, CursorPos::End);
                    ControlUI::Changed
                }
            }
            GoToPrevChar(anchor) => {
                if !anchor && self.value.is_anchored() {
                    let c = self.value.selection().start;
                    self.value.set_cursor(c, false);
                    ControlUI::Changed
                } else if self.value.cursor() == 0 {
                    ControlUI::Unchanged
                } else {
                    self.value.set_cursor(self.value.cursor() - 1, anchor);
                    ControlUI::Changed
                }
            }
            GoToNextChar(anchor) => {
                if !anchor && self.value.is_anchored() {
                    let c = self.value.selection().end;
                    self.value.set_cursor(c, false);
                    ControlUI::Changed
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::Unchanged
                } else {
                    self.value.set_cursor(self.value.cursor() + 1, anchor);
                    ControlUI::Changed
                }
            }
            GoToPrevWord(anchor) => {
                if self.value.cursor() == 0 {
                    ControlUI::Unchanged
                } else {
                    let cursor = self.value.prev_word_boundary();
                    self.value.set_cursor(cursor, anchor);
                    ControlUI::Changed
                }
            }
            GoToNextWord(anchor) => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::Unchanged
                } else {
                    let cursor = self.value.next_word_boundary();
                    self.value.set_cursor(cursor, anchor);
                    ControlUI::Changed
                }
            }
            DeleteLine => {
                if self.value.is_empty() {
                    ControlUI::Unchanged
                } else {
                    self.value.remove(0..self.value.len(), CursorPos::Start);
                    ControlUI::Changed
                }
            }
            DeletePrevWord => {
                if self.value.cursor() == 0 {
                    ControlUI::Unchanged
                } else {
                    let prev = self.value.prev_word_boundary();
                    self.value
                        .remove(prev..self.value.cursor(), CursorPos::Start);
                    ControlUI::Changed
                }
            }
            DeleteNextWord => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::Unchanged
                } else {
                    let next = self.value.next_word_boundary();
                    self.value.remove(self.value.cursor()..next, CursorPos::End);
                    ControlUI::Changed
                }
            }
            GoToStart(anchor) => {
                if self.value.cursor() == 0 {
                    ControlUI::Unchanged
                } else {
                    self.value.set_cursor(0, anchor);
                    ControlUI::Changed
                }
            }
            GoToEnd(anchor) => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::Unchanged
                } else {
                    self.value.set_cursor(self.value.len(), anchor);
                    ControlUI::Changed
                }
            }
            DeleteTillEnd => {
                self.value
                    .remove(self.value.cursor()..self.value.len(), CursorPos::Start);
                ControlUI::Changed
            }
            DeleteTillStart => {
                self.value.remove(0..self.value.cursor(), CursorPos::End);
                ControlUI::Changed
            }
            SelectAll => {
                self.value.set_cursor(0, false);
                self.value.set_cursor(self.value.len(), true);
                ControlUI::Changed
            }
        }
    }
}

pub mod core {
    #[allow(unused_imports)]
    use log::debug;
    use std::iter::once;
    use std::mem;
    use std::ops::Range;
    use unicode_segmentation::{Graphemes, UnicodeSegmentation};

    #[derive(Debug, PartialEq, Eq)]
    pub enum CursorPos {
        Start,
        End,
    }

    #[derive(Debug, Default, Clone)]
    pub struct InputMaskCore {
        // Input mask, coded.
        mask: String,
        // Input mask, translated for editing replacements.
        edit_mask: String,
        // Display mask for parts without useful value.
        display_mask: String,
        // Amalgamation for rendering
        render_string: String,
        // Base value.
        value: String,
        len: usize,

        offset: usize,
        width: usize,

        cursor: usize,
        anchor: usize,

        buf: String,
    }

    impl InputMaskCore {
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

        /// Width
        pub fn width(&self) -> usize {
            self.width
        }

        pub fn set_width(&mut self, width: usize) {
            self.width = width;

            if self.offset + width < self.cursor {
                self.offset = self.cursor - self.width;
            }
        }

        /// Cursor position as grapheme-idx.
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

        /// Cursor position as byte position.
        pub fn byte_cursor(&self) -> usize {
            let Some((i, _c)) = self
                .value
                .grapheme_indices(true)
                .chain(once((self.value.len(), "")))
                .skip(self.cursor)
                .next()
            else {
                unreachable!()
            };

            i
        }

        /// Set the mask that is shown.
        ///
        /// Panics:
        /// If the len differs from the mask.
        pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
            let display_mask = s.into();
            assert_eq!(
                self.mask.graphemes(true).count(),
                display_mask.graphemes(true).count()
            );
            self.display_mask = display_mask;
        }

        pub fn display_mask(&self) -> &str {
            self.display_mask.as_str()
        }

        /// Set the mask and generates a display mask.
        /// Uses the display mask as default value.
        pub fn set_mask<S: Into<String>>(&mut self, s: S) {
            self.mask = s.into();

            self.display_mask.clear();
            self.edit_mask.clear();
            for c in self.mask.chars() {
                self.display_mask.push(mask_value(c));
                self.edit_mask.push(mask_value(c));
            }

            self.set_value(self.edit_mask.clone());
        }

        /// Mask
        pub fn mask(&self) -> &str {
            self.mask.as_str()
        }

        /// Set the value. Resets cursor and anchor to 0.
        ///
        /// If the value doesn't conform to the given mask ... todo
        ///
        /// Panics
        /// If the len differs from the mask.
        pub fn set_value<S: Into<String>>(&mut self, s: S) {
            let value = s.into();
            let len = value.graphemes(true).count();

            assert_eq!(len, self.mask.graphemes(true).count());

            self.value = value;
            self.len = len;
            self.cursor = 0;
            self.offset = 0;
            self.anchor = 0;
        }

        /// value
        pub fn value(&self) -> &str {
            self.value.as_str()
        }

        /// Value with optional spaces removed.
        pub fn compact_value(&self) -> String {
            let mut s = String::new();
            for (c, m) in self.value.chars().zip(self.mask.chars()) {
                push_compact(c, m, &mut s);
            }
            s
        }

        /// Value that has been prepared with render_str().
        pub fn render_str(&self) -> &str {
            self.render_string.as_str()
        }

        /// Value to be rendered.
        ///
        /// Uses the value and the display-mask. If the value is space, the display-mask is used.
        /// It also uses the mask to create groups of characters. If one of the characters of a
        /// group has a value, the whole group is considered to have a value.
        ///
        pub fn render_value(&mut self) {
            let render = &mut self.render_string;
            render.clear();

            let mut mark_group = 'X';
            for (c, (m, d)) in self
                .value
                .chars()
                .zip(self.mask.chars().zip(self.display_mask.chars()))
            {
                match m {
                    '0' | 'H' | 'O' | 'L' | 'A' | 'C' | 'c' => {
                        mark_group = m;
                        render.push(c);
                    }
                    '9' | 'h' | 'o' | 'l' | 'a' | '#' | '_' => {
                        if c != ' ' {
                            mark_group = m;
                            render.push(c);
                        } else {
                            if mark_group == m {
                                render.push(' ');
                            } else {
                                mark_group = 'X';
                                render.push(d);
                            }
                        }
                    }
                    _ => {
                        mark_group = c;
                        render.push(d);
                    }
                }
            }
        }

        ///
        pub fn as_str(&self) -> &str {
            self.value.as_str()
        }

        /// graphemes
        pub fn graphemes(&self) -> Graphemes<'_> {
            self.value.graphemes(true)
        }

        /// clear
        pub fn clear(&mut self) {
            self.set_value("");
        }

        /// is_empty
        pub fn is_empty(&self) -> bool {
            self.value.is_empty()
        }

        /// value-len as grapheme-count
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

        pub fn selection(&self) -> Range<usize> {
            if self.cursor < self.anchor {
                self.cursor..self.anchor
            } else {
                self.anchor..self.cursor
            }
        }

        ///
        pub fn next_word_boundary(&self) -> usize {
            if self.cursor == self.len {
                self.len
            } else {
                self.value
                    .graphemes(true)
                    .enumerate()
                    .skip(self.cursor)
                    .skip_while(|(_, c)| is_alphanumeric(c))
                    .find(|(_, c)| is_alphanumeric(c))
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| self.len)
            }
        }

        ///
        pub fn prev_word_boundary(&self) -> usize {
            if self.cursor == 0 {
                0
            } else {
                self.value
                    .graphemes(true)
                    .rev()
                    .skip(self.len - self.cursor)
                    .skip_while(|c| !is_alphanumeric(c))
                    .skip_while(|c| is_alphanumeric(c))
                    .count()
            }
        }

        /// Insert a char, replacing the selection.
        pub fn insert_char(&mut self, new: char) {
            let selection = self.selection();

            let (_, mask, _) = split3(self.mask.as_str(), selection.start..self.len);
            if let Some(m) = mask.chars().next() {
                if is_valid_mask(new, m) {
                    self.remove(selection.clone(), CursorPos::Start);

                    let (before_str, _, after_str) =
                        split3(self.value.as_str(), selection.start..selection.start + 1);

                    self.buf.clear();
                    self.buf.push_str(before_str);
                    self.buf.push(new);
                    self.buf.push_str(after_str);

                    mem::swap(&mut self.value, &mut self.buf);

                    self.cursor += 1;
                    self.anchor = self.cursor;
                } else {
                    // skip to match
                    for (idx, c) in mask.chars().enumerate() {
                        if c == new {
                            self.cursor += idx + 1;
                            self.anchor = self.cursor;
                            break;
                        }
                    }
                }
            } else {
                // no more mask.
            }
        }

        /// Insert a string, replacing the selection.
        pub fn remove(&mut self, range: Range<usize>, cursor: CursorPos) {
            let (before_str, sel_str, after_str) = split3(self.value.as_str(), range.clone());
            let (_, sel_mask, _) = split3(self.edit_mask.as_str(), range.clone());

            let sel_len = sel_str.graphemes(true).count();
            let before_len = before_str.graphemes(true).count();

            if cursor == CursorPos::Start {
                self.cursor = before_len;
            } else {
                self.cursor = before_len + sel_len;
            }

            if self.offset > self.cursor {
                self.offset = self.cursor;
            } else if self.offset + self.width < self.cursor {
                self.offset = self.cursor - self.width;
            }

            self.anchor = self.cursor;

            self.buf.clear();
            self.buf.push_str(before_str);
            self.buf.push_str(sel_mask);
            self.buf.push_str(after_str);

            mem::swap(&mut self.value, &mut self.buf);
        }
    }

    fn push_compact(c: char, mask: char, buf: &mut String) {
        match mask {
            '0' | 'H' | 'O' | 'L' | 'A' | 'C' | 'c' | '_' => buf.push(c),
            '9' | 'h' | 'o' | 'l' | 'a' | '#' => {
                if c != ' ' {
                    buf.push(c);
                }
            }
            _ => buf.push(c),
        }
    }

    fn is_valid_mask(new: char, mask: char) -> bool {
        match mask {
            '0' => new.is_digit(10),
            '9' => new.is_digit(10) || new == ' ',
            'H' => new.is_digit(16),
            'h' => new.is_digit(16) || new == ' ',
            'O' => new.is_digit(8),
            'o' => new.is_digit(8) || new == ' ',
            'L' => new.is_alphabetic(),
            'l' => new.is_alphabetic() || new == ' ',
            'A' => new.is_alphanumeric(),
            'a' => new.is_alphanumeric() || new == ' ',
            'C' | 'c' => new.is_alphanumeric() || new == ' ',
            '#' => new.is_digit(10) || new == ' ' || new == '+' || new == '-',
            '_' => true,
            _ => mask == new,
        }
    }

    fn mask_value(mask: char) -> char {
        match mask {
            '0' => '0',
            '9' => ' ',
            'H' => '0',
            'h' => ' ',
            'O' => '0',
            'o' => ' ',
            '#' => ' ',
            'L' => 'X',
            'l' => ' ',
            'A' => 'X',
            'a' => ' ',
            'C' => ' ',
            'c' => ' ',
            '_' => ' ',
            _ => mask,
        }
    }

    /// Split off selection
    pub fn split3(value: &str, selection: Range<usize>) -> (&str, &str, &str) {
        let mut byte_selection_start = None;
        let mut byte_selection_end = None;

        for (cidx, (idx, _c)) in value
            .grapheme_indices(true)
            .chain(once((value.len(), "")))
            .enumerate()
        {
            if cidx == selection.start {
                byte_selection_start = Some(idx);
            }
            if cidx == selection.end {
                byte_selection_end = Some(idx)
            }
        }

        let byte_selection_start = byte_selection_start.expect("byte_selection_start_not_found");
        let byte_selection_end = byte_selection_end.expect("byte_selection_end_not_found");

        let before_str = &value[0..byte_selection_start];
        let sel_str = &value[byte_selection_start..byte_selection_end];
        let after_str = &value[byte_selection_end..value.len()];

        (before_str, sel_str, after_str)
    }

    /// Split off selection and cursor
    pub fn split5(
        value: &str,
        cursor: usize,
        visible: Range<usize>,
        selection: Range<usize>,
    ) -> (&str, &str, &str, &str, &str) {
        let mut vis_sta = None;
        let mut vis_end = None;
        let mut sel_sta = None;
        let mut sel_end = None;
        let mut cur_sta = None;
        let mut cur_len = None;

        for (cidx, (idx, c)) in value
            .grapheme_indices(true)
            .chain(once((value.len(), "")))
            .enumerate()
        {
            if cidx == visible.start {
                vis_sta = Some(idx);
            }
            if cidx == visible.end {
                vis_end = Some(idx);
            }
            if cidx == selection.start {
                sel_sta = Some(idx);
            }
            if cidx == selection.end {
                sel_end = Some(idx);
            }
            if cidx == cursor {
                cur_sta = Some(idx);
                cur_len = Some(c.len())
            }
        }

        let vis_sta = vis_sta.expect("visible_start_not_found");
        let vis_end = vis_end.expect("visible_end_not_found");
        let sel_sta = sel_sta.expect("selection_start_not_found");
        let sel_end = sel_end.expect("selection_end_not_found");
        let cur_sta = cur_sta.expect("cursor_start_not_found");
        let cur_len = cur_len.expect("cursor_end_not_found");

        let before_str = &value[vis_sta..sel_sta];

        let (cursor1_str, sel_str) = if sel_sta == cur_sta && sel_sta + cur_len <= sel_end {
            (
                &value[cur_sta..cur_sta + cur_len],
                &value[sel_sta + cur_len..sel_end],
            )
        } else {
            (&value[sel_sta..sel_sta], &value[sel_sta..sel_end])
        };

        let (cursor2_str, after_str) = if cur_len == 0 {
            (" ", &value[sel_end..vis_end])
        } else if sel_end == cur_sta && sel_end + cur_len <= vis_end {
            (
                &value[cur_sta..cur_sta + cur_len],
                &value[sel_end + cur_len..vis_end],
            )
        } else {
            (&value[sel_end..sel_end], &value[sel_end..vis_end])
        };

        (before_str, cursor1_str, sel_str, cursor2_str, after_str)
    }

    fn is_alphanumeric(s: &str) -> bool {
        if let Some(c) = s.chars().next() {
            c.is_alphanumeric()
        } else {
            false
        }
    }
}
