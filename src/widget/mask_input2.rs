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
use crate::{ControlUI, ValidFlag};
use crate::{DefaultKeys, FrameWidget, HandleCrossterm, Input, MouseOnly};
use crate::{FocusFlag, HasFocusFlag, HasValidFlag};
use core::{split3, split5, CursorPos};
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
        self.value.reset();
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
    pub fn display_mask(&self) -> String {
        self.value.display_mask()
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below. See [compact_value()](MaskedInputState::compact_value).
    ///
    /// * 0: can enter digit, display as 0 TODO: change to can? remove leading 0?
    /// * 9: can enter digit, display as space
    /// * #: digit, plus or minus sign, display as space
    /// TODO extend with . and , for full numeric input
    ///
    /// * H: must enter a hex digit, display as 0
    /// * h: can enter a hex digit, display as space
    /// * O: must enter an octal digit, display as 0
    /// * o: can enter an octal digit, display as space
    /// * D: must enter a decimal digit, display as 0
    /// * d: can enter a decimal digit, display as space
    ///
    /// * L: must enter letter, display as X
    /// * l: can enter letter, display as space
    /// * A: must enter letter or digit, display as X
    /// * a: can enter letter or digit, display as space
    /// * C: must enter character or space, display as space
    /// * c: can enter character or space, display as space
    /// * _: anything, display as space
    ///
    /// * . , : ; - /: grouping characters move the cursor when entered
    ///
    /// * use \ to escape any of the above. TODO
    ///
    /// Inspired by <https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da>
    pub fn set_mask<S: Into<String>>(&mut self, s: S) {
        self.value.set_mask(s);
    }

    /// Display mask.
    pub fn mask(&self) -> String {
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
            self.value.rendered(),
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
                self.value.skip_cursor();
                self.value.advance_cursor(c);
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
    use log::debug;
    use std::fmt::{Debug, Formatter};
    use std::iter::once;
    use std::ops::Range;
    use unicode_segmentation::UnicodeSegmentation;

    #[derive(Debug, PartialEq, Eq)]
    pub enum CursorPos {
        Start,
        End,
    }

    #[derive(Clone)]
    pub struct MaskToken {
        pub sec_nr: usize,
        pub sec_start: usize,
        pub sec_end: usize,

        pub peek_left: Box<str>,
        pub right: Box<str>,
        pub edit: Box<str>,
        pub display: Box<str>,
    }

    impl Debug for MaskToken {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Mask {}:{}-{} {}<|{}|",
                self.sec_nr, self.sec_start, self.sec_end, self.peek_left, self.right
            )
        }
    }

    impl MaskToken {
        fn skip_start(mask: &[MaskToken], v: &str) -> usize {
            let mut skip = 0;
            for (c, m) in v.graphemes(true).zip(mask.iter()) {
                if !MaskToken::can_drop(&m.right, c) {
                    break;
                } else {
                    skip += 1;
                }
            }
            skip
        }

        fn skip_end(mask: &[MaskToken], v: &str) -> usize {
            let mut skip = 0;
            for (c, m) in v.graphemes(true).rev().zip(mask.iter().rev()) {
                if !MaskToken::can_drop(&m.right, c) {
                    break;
                } else {
                    skip += 1;
                }
            }
            skip
        }

        fn is_ltor(mask: &str) -> bool {
            let Some(mask) = mask.chars().next() else {
                return false;
            };
            match mask {
                '0' => false,
                '9' => false,
                '#' => false,

                'H' => true,
                'h' => true,
                'O' => true,
                'o' => true,
                'D' => true,
                'd' => true,

                'L' => true,
                'l' => true,
                'A' => true,
                'a' => true,
                'C' => true,
                'c' => true,
                '_' => true,

                _ => true,
            }
        }

        fn is_rtol(mask: &str) -> bool {
            let Some(mask) = mask.chars().next() else {
                return false;
            };
            match mask {
                '0' => true,
                '9' => true,
                '#' => true,

                'H' => false,
                'h' => false,
                'O' => false,
                'o' => false,
                'D' => false,
                'd' => false,

                'L' => false,
                'l' => false,
                'A' => false,
                'a' => false,
                'C' => false,
                'c' => false,
                '_' => false,
                _ => false,
            }
        }

        fn mask_type(mask: &str) -> u8 {
            let Some(mask) = mask.chars().next() else {
                return 99;
            };
            match mask {
                '0' => 0,
                '9' => 0,
                '#' => 0,

                'H' => 1,
                'h' => 1,
                'O' => 1,
                'o' => 1,
                'D' => 1,
                'd' => 1,

                'L' => 2,
                'l' => 2,
                'A' => 2,
                'a' => 2,
                'C' => 2,
                'c' => 2,
                '_' => 2,

                _ => 3,
            }
        }

        fn shall_overwrite(mask: &str) -> bool {
            let mask = mask.chars().next().expect("mask");
            match mask {
                '0' | '9' | 'h' | 'o' | 'd' | 'l' | 'a' | 'c' | '_' | '#' => false,
                'H' | 'O' | 'D' | 'L' | 'A' | 'C' | _ => true,
            }
        }

        fn can_drop(mask: &str, c: &str) -> bool {
            let mask = mask.chars().next().expect("mask");
            match mask {
                '9' | 'h' | 'o' | 'd' | 'l' | 'a' | 'c' | '_' | '#' => {
                    if c == " " {
                        true
                    } else {
                        false
                    }
                }
                '0' => {
                    if c == "0" {
                        true
                    } else {
                        false
                    }
                }
                'H' | 'O' | 'D' | 'L' | 'A' | 'C' | _ => false,
            }
        }

        fn can_skip(mask: &str, c: &str) -> bool {
            let mask = mask.chars().next().expect("mask");
            match mask {
                '9' | 'h' | 'o' | 'd' | 'l' | 'a' | 'c' | '_' | '#' => {
                    if c == " " {
                        true
                    } else {
                        false
                    }
                }
                '0' | 'H' | 'O' | 'D' | 'L' | 'A' | 'C' | _ => false,
            }
        }

        fn is_sep_char(mask: &str, test: &str) -> bool {
            let Some(mask) = mask.chars().next() else {
                return false;
            };
            let Some(test) = test.chars().next() else {
                return false;
            };
            match mask {
                '0' => false,
                '9' => false,
                '#' => false,

                'H' => false,
                'h' => false,
                'O' => false,
                'o' => false,
                'D' => false,
                'd' => false,

                'L' => false,
                'l' => false,
                'A' => false,
                'a' => false,
                'C' => false,
                'c' => false,
                '_' => false,

                c => c == test,
            }
        }

        fn is_valid_char(mask: &str, test: &str) -> bool {
            let Some(mask) = mask.chars().next() else {
                return false;
            };
            let Some(test) = test.chars().next() else {
                return false;
            };
            match mask {
                '0' => test.is_ascii_digit(),
                '9' => test.is_ascii_digit() || test == ' ',
                '#' => test.is_ascii_digit() || test == ' ' || test == '+' || test == '-',

                'H' => test.is_ascii_hexdigit(),
                'h' => test.is_ascii_hexdigit() || test == ' ',
                'O' => test.is_digit(8),
                'o' => test.is_digit(8) || test == ' ',
                'D' => test.is_ascii_digit(),
                'd' => test.is_ascii_digit() || test == ' ',

                'L' => test.is_alphabetic(),
                'l' => test.is_alphabetic() || test == ' ',
                'A' => test.is_alphanumeric(),
                'a' => test.is_alphanumeric() || test == ' ',
                'C' => test.is_alphanumeric() || test == ' ',
                'c' => test.is_alphanumeric() || test == ' ',

                '_' => true,
                _ => false,
            }
        }

        fn edit_value(mask: &str) -> &str {
            match mask {
                "0" => "0",
                "9" => " ",
                "#" => " ",

                "H" => "0",
                "h" => " ",
                "O" => "0",
                "o" => " ",
                "D" => "0",
                "d" => " ",

                "L" => " ",
                "l" => " ",
                "A" => " ",
                "a" => " ",
                "C" => " ",
                "c" => " ",
                "_" => " ",
                _ => mask,
            }
        }

        fn disp_value(mask: &str) -> &str {
            match mask {
                "0" => "0",
                "9" => " ",
                "#" => " ",

                "H" => "0",
                "h" => " ",
                "O" => "0",
                "o" => " ",
                "D" => "0",
                "d" => " ",

                "L" => "X",
                "l" => " ",
                "A" => "X",
                "a" => " ",
                "C" => " ",
                "c" => " ",
                "_" => " ",
                _ => mask,
            }
        }
    }

    /// Text editing core.
    #[derive(Debug, Default, Clone)]
    pub struct InputMaskCore {
        mask: Vec<MaskToken>,
        value: String,
        rendered: String,
        len: usize,

        offset: usize,
        width: usize,

        cursor: usize,
        anchor: usize,
    }

    impl InputMaskCore {
        /// Reset value but not the mask and width
        pub fn reset(&mut self) {
            self.value = String::new();
            self.len = 0;
            self.offset = 0;
            // width not reset
            self.cursor = 0;
            self.anchor = 0;
        }

        /// Offset
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Change the offset.
        ///
        /// Ensures the cursor is visible and modifies any given offset.
        /// Ensures the offset is not beyond the length.
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
        pub fn cursor(&self) -> usize {
            self.cursor
        }

        pub fn anchor(&self) -> usize {
            self.anchor
        }

        pub fn is_anchored(&self) -> bool {
            self.cursor != self.anchor
        }

        pub fn selection(&self) -> Range<usize> {
            if self.cursor < self.anchor {
                self.cursor..self.anchor
            } else {
                self.anchor..self.cursor
            }
        }

        pub fn set_cursor(&mut self, cursor: usize, anchor: bool) {
            if cursor > self.len {
                self.cursor = self.len;
            } else {
                self.cursor = cursor;
            }

            if !anchor {
                self.anchor = self.cursor;
            }

            if self.offset > self.cursor {
                self.offset = self.cursor;
            } else if self.offset + self.width < self.cursor {
                self.offset = self.cursor - self.width;
            }
        }

        pub fn set_mask<S: Into<String>>(&mut self, s: S) {
            let mask = s.into();

            self.mask.clear();

            let mut nr = 0;
            let mut start = 0;
            let mut last_m = None;
            for (i, m) in mask.graphemes(true).chain(once("")).enumerate() {
                if let Some(last_m) = last_m {
                    if MaskToken::mask_type(m) != MaskToken::mask_type(last_m) {
                        for j in start..i {
                            self.mask[j].sec_nr = nr;
                            self.mask[j].sec_start = start;
                            self.mask[j].sec_end = i;
                        }

                        nr += 1;
                        start = i;
                    }
                }

                let tok = MaskToken {
                    sec_nr: 0,
                    sec_start: 0,
                    sec_end: 0,
                    peek_left: Box::from(last_m.unwrap_or_default()),
                    right: Box::from(m),
                    edit: MaskToken::edit_value(m).into(),
                    display: MaskToken::disp_value(m).into(),
                };
                self.mask.push(tok);

                last_m = Some(m);
            }
            for j in start..self.mask.len() {
                self.mask[j].sec_nr = nr;
                self.mask[j].sec_start = start;
                self.mask[j].sec_end = mask.graphemes(true).count();
            }

            let mut buf = String::new();
            for t in &self.mask {
                buf.push_str(&t.edit);
            }
            self.set_value(buf);
        }

        pub fn mask(&self) -> String {
            let mut buf = String::new();
            for t in self.mask.iter() {
                buf.push_str(t.right.as_ref())
            }
            buf
        }

        /// Set the mask that is shown.
        ///
        /// Panics:
        /// If the len differs from the mask.
        pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
            let display_mask = s.into();

            for (t, m) in self.mask.iter_mut().zip(display_mask.graphemes(true)) {
                t.display = Box::from(m);
            }
        }

        /// Display mask
        pub fn display_mask(&self) -> String {
            let mut buf = String::new();
            for t in &self.mask {
                buf.push_str(&t.display);
            }
            buf
        }

        pub fn set_value<S: Into<String>>(&mut self, s: S) {
            let value = s.into();
            let len = value.graphemes(true).count();

            assert_eq!(len, self.mask.len() - 1);

            self.value = value;
            self.len = len;

            if self.offset > self.len {
                self.offset = self.len;
            }
            if self.cursor > self.len {
                self.cursor = self.len;
            }
        }

        pub fn value(&self) -> &str {
            self.value.as_str()
        }

        pub fn compact_value(&self) -> String {
            let mut buf = String::new();
            for (c, m) in self.value.graphemes(true).zip(self.mask.iter()) {
                if !MaskToken::can_skip(&m.right, c) {
                    buf.push_str(c);
                }
            }
            buf
        }

        pub fn is_empty(&self) -> bool {
            for (m, c) in self.mask.iter().zip(self.value.graphemes(true)) {
                if c != m.edit.as_ref() {
                    return false;
                }
            }
            true
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn rendered(&self) -> &str {
            self.rendered.as_str()
        }

        pub fn render_value(&mut self) {
            self.rendered.clear();

            let buf = &mut self.rendered;
            let mut sec_nr = 0;
            let mut sec_filled = false;
            for (c, m) in self.value.graphemes(true).zip(self.mask.iter()) {
                if sec_nr != m.sec_nr {
                    sec_filled = false;
                    sec_nr = m.sec_nr;
                }
                sec_filled |= !MaskToken::can_skip(&m.right, c);

                // todo!
                // if !sec_filled && MaskToken::can_skip(&m.right, c) {
                //     buf.push_str(&m.display);
                // } else {
                buf.push_str(c);
                // }
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
                    .skip_while(|c| is_alphanumeric(c))
                    .skip_while(|c| is_alphanumeric(c))
                    .count()
            }
        }

        /// Sets the cursor to a position where a character may be set.
        pub fn skip_cursor(&mut self) {
            let mask = &self.mask[self.cursor];

            if MaskToken::is_rtol(&mask.right) {
                let (_b, _c0, c1, _a) = self.split_mask(mask);
                let submask = &self.mask[self.cursor..mask.sec_end];
                let skip = MaskToken::skip_start(submask, c1);
                self.cursor += skip;
                self.anchor = self.cursor;
            } else if MaskToken::is_ltor(&mask.right) {
                let (_b, c0, _c1, _a) = self.split_mask(mask);
                let submask = &self.mask[mask.sec_start..self.cursor];
                let skip = MaskToken::skip_end(submask, c0);
                self.cursor -= skip;
                self.anchor = self.cursor;
            }
        }

        // Clear the section if the cursor is at the very first position.
        pub fn clear_section(&mut self) {
            let mask = &self.mask[self.cursor];

            if MaskToken::is_rtol(&mask.peek_left) {
                // noop
            } else if MaskToken::is_rtol(&mask.right) {
                let (b, c0, _c1, a) = self.split_mask(mask);
                if c0.is_empty() {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(self.empty_section(mask).as_str());
                    buf.push_str(a);
                    self.value = buf;

                    // cursor stays
                }
            } else if MaskToken::is_ltor(&mask.right) {
                let (b, c0, _c1, a) = self.split_mask(mask);
                if c0.is_empty() {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(self.empty_section(mask).as_str());
                    buf.push_str(a);
                    self.value = buf;

                    // cursor stays
                }
            } else if MaskToken::is_ltor(&mask.peek_left) {
                // noop
            }
        }

        // Advance the cursor to the next section, if new char matches
        // certain conditions.
        pub fn advance_cursor(&mut self, c: char) {
            let buf = String::from(c);
            let cc = buf.as_str();

            let mask = &self.mask[self.cursor];

            if MaskToken::is_rtol(&mask.peek_left) {
                let mask = &self.mask[self.cursor - 1];
                let mask0 = &self.mask[mask.sec_start];
                let (_b, c0, _c1, _a) = self.split_mask(mask);
                // section is full or char invalid
                if can_drop_first(&mask0.right, c0)
                    && (MaskToken::is_valid_char(&mask.right, cc)
                        || MaskToken::is_sep_char(&mask.right, cc))
                {
                    // insert char later.
                } else {
                    self.cursor = self.find_match(cc);
                    self.anchor = self.cursor;
                }
            } else if MaskToken::is_ltor(&mask.right) {
                let mask9 = &self.mask[mask.sec_end - 1];
                let (_b, _c0, c1, _a) = self.split_mask(mask);

                if can_drop_last(&mask9.right, c1)
                    && (MaskToken::is_valid_char(&mask.right, cc)
                        || MaskToken::is_sep_char(&mask.right, cc))
                {
                    // insert char later
                } else {
                    self.cursor = self.find_match(cc);
                    self.anchor = self.cursor;
                }
            } else if MaskToken::is_rtol(&mask.right) {
                // left end of rtol. advance
                self.cursor = self.find_match(cc);
                self.anchor = self.cursor;
            }
        }

        // Insert the char if it matches the cursor mask and the current
        // section is not full.
        pub fn insert_char(&mut self, c: char) {
            let buf = String::from(c);
            let cc = buf.as_str();

            let mask = &self.mask[self.cursor];

            if MaskToken::is_rtol(&mask.peek_left) {
                let mask = &self.mask[self.cursor - 1];
                let mask0 = &self.mask[mask.sec_start];

                let (b, c0, c1, a) = self.split_mask(mask);
                if can_drop_first(&mask0.right, c0)
                    && (MaskToken::is_valid_char(&mask.right, cc)
                        || MaskToken::is_sep_char(&mask.right, cc))
                {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(drop_first(c0));
                    buf.push_str(cc);
                    buf.push_str(c1);
                    buf.push_str(a);
                    self.value = buf;

                    // cursor stays
                    return;
                }
            }
            if MaskToken::is_ltor(&mask.right) {
                let mask9 = &self.mask[mask.sec_end - 1];

                let (b, c0, c1, a) = self.split_mask(mask);
                if MaskToken::shall_overwrite(&mask.right)
                    && (MaskToken::is_valid_char(&mask.right, cc)
                        || MaskToken::is_sep_char(&mask.right, cc))
                {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(c0);
                    buf.push_str(cc);
                    buf.push_str(drop_first(c1));
                    buf.push_str(a);
                    self.value = buf;

                    self.cursor += 1;
                    self.anchor = self.cursor;
                    return;
                }
                if can_drop_last(&mask9.right, c1)
                    && (MaskToken::is_valid_char(&mask.right, cc)
                        || MaskToken::is_sep_char(&mask.right, cc))
                {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(c0);
                    buf.push_str(cc);
                    buf.push_str(drop_last(c1));
                    buf.push_str(a);
                    self.value = buf;

                    self.cursor += 1;
                    self.anchor = self.cursor;
                    return;
                }
            }
        }

        /// Insert a string, replacing the selection.
        pub fn remove(&mut self, range: Range<usize>, cursor: CursorPos) {
            let (before_str, sel_str, after_str) = split3(self.value.as_str(), range.clone());
            let mask = &self.mask[range];

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

            let mut buf = String::new();
            buf.push_str(before_str);
            for m in mask {
                buf.push_str(&m.edit);
            }
            buf.push_str(after_str);

            self.value = buf;
        }

        /// Create a string with the default edit mask.
        fn empty_section(&self, mask: &MaskToken) -> String {
            let mut buf = String::new();
            for i in mask.sec_start..mask.sec_end {
                let m = &self.mask[i];
                buf.push_str(&m.edit);
            }
            buf
        }

        /// Find a possible match in the mask.
        fn find_match(&self, cc: &str) -> usize {
            for i in self.cursor..self.mask.len() {
                let test = &self.mask[i];
                if MaskToken::is_sep_char(&test.right, cc) {
                    return i + 1;
                } else if MaskToken::is_valid_char(&test.right, cc) {
                    return i;
                }
            }
            self.cursor
        }

        /// Split along mask-sections
        fn split_mask(&self, mask: &MaskToken) -> (&str, &str, &str, &str) {
            let value = &self.value;

            let mut byte_mask_start = None;
            let mut byte_mask_end = None;
            let mut byte_cursor = None;

            for (cidx, (idx, _c)) in value
                .grapheme_indices(true)
                .chain(once((value.len(), "")))
                .enumerate()
            {
                if cidx == self.cursor {
                    byte_cursor = Some(idx);
                }
                if cidx == mask.sec_start {
                    byte_mask_start = Some(idx);
                }
                if cidx == mask.sec_end {
                    byte_mask_end = Some(idx);
                }
            }

            let byte_cursor = byte_cursor.expect("mask");
            let byte_mask_start = byte_mask_start.expect("mask");
            let byte_mask_end = byte_mask_end.expect("mask");

            (
                &value[..byte_mask_start],
                &value[byte_mask_start..byte_cursor],
                &value[byte_cursor..byte_mask_end],
                &value[byte_mask_end..],
            )
        }
    }

    fn drop_first(s: &str) -> &str {
        if s.is_empty() {
            unreachable!();
        } else {
            let (_, f, r) = split3(s, 0..1);
            r
        }
    }

    fn drop_last(s: &str) -> &str {
        if s.is_empty() {
            unreachable!();
        } else {
            let end = s.graphemes(true).count();
            let (r, f, _) = split3(s, end - 1..end);
            r
        }
    }

    fn can_drop_first(mask: &str, s: &str) -> bool {
        if s.is_empty() {
            false
        } else {
            let (_, f, _) = split3(s, 0..1);
            MaskToken::can_drop(mask, f)
        }
    }

    fn can_drop_last(mask: &str, s: &str) -> bool {
        if s.is_empty() {
            false
        } else {
            let end = s.graphemes(true).count();
            let (_, f, _) = split3(s, end - 1..end);
            MaskToken::can_drop(mask, f)
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

    #[cfg(test)]
    mod tests {
        use crate::widget::mask_input2::core::InputMaskCore;

        #[test]
        fn test_mask() {
            let mut c = InputMaskCore::default();
            c.set_mask("999.999");
            dbg!("{:?}", c);
        }
    }
}
