//! Text input with an input mask.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection
//! * Scrolls with the cursor.
//! * Can set the cursor or use its own block cursor.
//! * Can show an indicator for invalid input.
//!
//! * Accepts an input mask
//! * Accepts a display overlay used instead of the default chars of the input mask.
//!

use crate::widget::basic::ClearStyle;
use crate::widget::mask_input::core::{split3, split5, CursorPos};
use crate::{ControlUI, ValidFlag};
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

/// Text input widget with input mask.
#[derive(Debug)]
pub struct MaskedInput {
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

/// Combined style.
#[derive(Debug, Default)]
pub struct MaskedInputStyle {
    pub style: Style,
    pub focus: Style,
    pub select: Style,
    pub cursor: Option<Style>,
    pub invalid: Option<Style>,
}

impl Default for MaskedInput {
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

impl MaskedInput {
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
    pub fn style(mut self, style: MaskedInputStyle) -> Self {
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

impl FrameWidget for MaskedInput {
    type State = MaskedInputState;

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
        if focus {
            frame.set_cursor(l_input.x + state.visible_cursor(), l_input.y);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MaskedInputState {
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
    /// Editing core.
    pub value: core::InputMaskCore,
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for MaskedInputState {
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

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for MaskedInputState {
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
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

/// Mapping from events to abstract editing requests.
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

impl MaskedInputState {
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

    /// Cursor position
    pub fn cursor(&self) -> usize {
        self.value.cursor()
    }

    /// Set the display mask. This text is used for parts that have
    /// no valid input yet. Part means consecutive characters of the
    /// input mask with the same mask type.
    ///
    /// There is a default representation for each mask type if this
    /// is not set.
    ///
    /// Panic
    /// Panics if the length differs from the  mask.
    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.value.set_display_mask(s);
    }

    /// Display mask.
    pub fn display_mask(&self) -> &str {
        self.value.display_mask()
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below. See [compact_value()](crate::widget::mask_input::MaskedInputState::compact_value).
    ///
    /// * 0: must enter digit, display as 0
    /// * 9: can enter digit, display as space
    /// * H: must enter a hex digit, display as 0
    /// * h: can enter a hex digit, display as space
    /// * O: must enter an octal digit, display as 0
    /// * o: can enter an octal digit, display as space
    /// * L: must enter letter, display as X
    /// * l: can enter letter, display as space
    /// * A: must enter letter or digit, display as X
    /// * a: can enter letter or digit, display as space
    /// * C: must enter character or space, display as space
    /// * c: can enter character or space, display as space
    /// * _: anything, display as space
    /// * #: digit, plus or minus sign, display as space
    /// * . , : ; - /: grouping characters move the cursor when entered
    ///
    /// Inspired by <https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da>
    pub fn set_mask<S: Into<String>>(&mut self, s: S) {
        self.value.set_mask(s);
    }

    /// Display mask.
    pub fn mask(&self) -> &str {
        self.value.mask()
    }

    /// Set the value.
    ///
    /// Panic
    /// Panics if the grapheme length of the value is not the same as the mask.
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.value.set_value(s);
    }

    /// Value with all punctuation and default values according to the mask type.
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

    /// Value.
    pub fn as_str(&self) -> &str {
        self.value.value()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Length in grapheme count.
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Selection
    pub fn has_selection(&self) -> bool {
        self.value.is_anchored()
    }

    /// Selection
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.value.set_cursor(anchor, false);
        self.value.set_cursor(cursor, true);
    }

    /// Selection
    pub fn select_all(&mut self) {
        // the other way round it fails if width is 0.
        self.value.set_cursor(self.value.len(), false);
        self.value.set_cursor(0, true);
    }

    /// Selection
    pub fn selection(&self) -> Range<usize> {
        self.value.selection()
    }

    /// Selection
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
}

impl HasFocusFlag for MaskedInputState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasValidFlag for MaskedInputState {
    fn valid(&self) -> &ValidFlag {
        &self.valid
    }
}

impl<A, E> Input<ControlUI<A, E>> for MaskedInputState {
    type Request = InputRequest;

    fn perform(&mut self, action: Self::Request) -> ControlUI<A, E> {
        use InputRequest::*;

        match action {
            SetCursor(pos, anchor) => {
                let pos = pos + self.value.offset();
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
                    self.value.remove(self.value.selection(), CursorPos::Start);
                    ControlUI::Change
                } else if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value.remove(
                        self.value.cursor() - 1..self.value.cursor(),
                        CursorPos::Start,
                    );
                    ControlUI::Change
                }
            }
            DeleteNextChar => {
                if self.value.is_anchored() {
                    self.value.remove(self.value.selection(), CursorPos::End);
                    ControlUI::Change
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value
                        .remove(self.value.cursor()..self.value.cursor() + 1, CursorPos::End);
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
                    self.value.remove(0..self.value.len(), CursorPos::Start);
                    ControlUI::Change
                }
            }
            DeletePrevWord => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    let prev = self.value.prev_word_boundary();
                    self.value
                        .remove(prev..self.value.cursor(), CursorPos::Start);
                    ControlUI::Change
                }
            }
            DeleteNextWord => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    let next = self.value.next_word_boundary();
                    self.value.remove(self.value.cursor()..next, CursorPos::End);
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
                    .remove(self.value.cursor()..self.value.len(), CursorPos::Start);
                ControlUI::Change
            }
            DeleteTillStart => {
                self.value.remove(0..self.value.cursor(), CursorPos::End);
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

pub mod core {
    #[allow(unused_imports)]
    use log::debug;
    use std::iter::once;
    use std::mem;
    use std::ops::Range;
    use unicode_segmentation::{Graphemes, UnicodeSegmentation};

    /// Indicates where the cursor should be placed after a remove().
    #[derive(Debug, PartialEq, Eq)]
    pub enum CursorPos {
        Start,
        End,
    }

    /// Text editing core.
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
        // Len in grapheme count.
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
                .nth(self.cursor)
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

        /// Display mask
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
        /// If the value doesn't conform to the given mask it is so. ... todo?
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
                    self.do_insert_char(new);
                } else if self.cursor == self.anchor {
                    let skip_to_sep = mask
                        .chars()
                        .enumerate()
                        .find(|(_i, m)| is_valid_sep(new, *m));
                    if let Some((skip, _m)) = skip_to_sep {
                        self.cursor += skip + 1;
                        self.anchor = self.cursor;
                    }

                    let skip_over_sep = mask.chars().enumerate().find(|(_, m)| is_sep(*m));
                    if let Some((skip, mm)) = skip_over_sep {
                        if is_valid_mask(new, mm) {
                            self.cursor += skip;
                            self.anchor = self.cursor;
                            self.do_insert_char(new);
                        }
                    }
                }
            } else {
                // no more mask.
            }
        }

        fn do_insert_char(&mut self, new: char) {
            let selection = self.selection();

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

    fn is_sep(mask: char) -> bool {
        matches!(
            mask,
            '0' | '9' | 'H' | 'h' | 'O' | 'o' | 'L' | 'l' | 'A' | 'a' | 'C' | 'c' | '_'
        )
    }

    fn is_valid_sep(new: char, mask: char) -> bool {
        is_sep(mask) && new == mask
    }

    fn is_valid_mask(new: char, mask: char) -> bool {
        match mask {
            '0' => new.is_ascii_digit(),
            '9' => new.is_ascii_digit() || new == ' ',
            'H' => new.is_ascii_hexdigit(),
            'h' => new.is_ascii_hexdigit() || new == ' ',
            'O' => new.is_digit(8),
            'o' => new.is_digit(8) || new == ' ',
            'L' => new.is_alphabetic(),
            'l' => new.is_alphabetic() || new == ' ',
            'A' => new.is_alphanumeric(),
            'a' => new.is_alphanumeric() || new == ' ',
            'C' | 'c' => new.is_alphanumeric() || new == ' ',
            '#' => new.is_ascii_digit() || new == ' ' || new == '+' || new == '-',
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
